//!
//! Translates the calldata instructions.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the calldata load.
///
pub fn load<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 1],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let if_zero_block = context.append_basic_block("calldata_if_zero");
    let if_non_zero_block = context.append_basic_block("calldata_if_not_zero");
    let join_block = context.append_basic_block("calldata_if_join");

    let value_pointer = context.build_alloca(context.field_type(), "calldata_value_pointer");
    context.build_store(value_pointer, context.field_const(0));
    let is_zero = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        arguments[0].into_int_value(),
        context.field_const(0),
        "calldata_if_zero_condition",
    );
    context.build_conditional_branch(is_zero, if_zero_block, if_non_zero_block);

    context.set_basic_block(if_zero_block);
    let offset = context.field_const(
        (compiler_common::abi::OFFSET_ENTRY_DATA * compiler_common::size::FIELD) as u64,
    );
    let pointer = context.access_memory(
        offset,
        compiler_common::AddressSpace::Parent,
        "hash_pointer",
    );
    let value = context.build_load(pointer, "calldata_entry_hash_value");
    context.build_store(value_pointer, value);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(if_non_zero_block);
    let offset = context.builder.build_int_add(
        arguments[0].into_int_value(),
        context.field_const(
            (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD - 4)
                as u64,
        ),
        "calldata_value_offset",
    );
    let pointer = context.access_memory(
        offset,
        compiler_common::AddressSpace::Parent,
        "pointer_non_zero",
    );
    let value = context.build_load(pointer, "calldata_value_non_zero");
    context.build_store(value_pointer, value);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let value = context.build_load(value_pointer, "calldata_value_result");
    Some(value)
}

///
/// Translates the calldata size.
///
pub fn size<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    has_selector: bool,
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let pointer = context.access_memory(
        context.field_const(
            (compiler_common::abi::OFFSET_CALLDATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        compiler_common::AddressSpace::Parent,
        "calldata_size_pointer",
    );
    let value = context.build_load(pointer, "calldata_size_value_cells");
    let mut value = context.builder.build_int_mul(
        value.into_int_value(),
        context.field_const(compiler_common::size::FIELD as u64),
        "calldata_size_value_bytes",
    );
    if has_selector {
        value = context.builder.build_int_add(
            value,
            context.field_const(4),
            "calldata_size_value_bytes_with_selector",
        );
    }
    Some(value.as_basic_value_enum())
}

///
/// Translates the calldata copy.
///
pub fn copy<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let copy_block = context.append_basic_block("calldata_if_copy");
    let zero_block = context.append_basic_block("calldata_if_zero");
    let join_block = context.append_basic_block("calldata_if_join");

    let pointer = context.access_memory(
        context.field_const(
            (compiler_common::abi::OFFSET_CALLDATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        compiler_common::AddressSpace::Parent,
        "calldata_size_pointer",
    );
    let calldata_size = context
        .build_load(pointer, "calldata_size_value_cells")
        .into_int_value();

    let range_end_bytes = context.builder.build_int_add(
        arguments[1].into_int_value(),
        arguments[2].into_int_value(),
        "calldata_range_end_bytes",
    );
    let range_end = context.builder.build_int_unsigned_div(
        range_end_bytes,
        context.field_const(compiler_common::size::FIELD as u64),
        "calldata_range_end",
    );

    let is_calldata_available = context.builder.build_int_compare(
        inkwell::IntPredicate::UGE,
        calldata_size,
        range_end,
        "calldata_is_available",
    );
    context.build_conditional_branch(is_calldata_available, copy_block, zero_block);

    context.set_basic_block(copy_block);
    let destination = context.access_memory(
        arguments[0].into_int_value(),
        compiler_common::AddressSpace::Heap,
        "calldata_copy_destination_pointer",
    );

    let source_offset_shift =
        compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD - 4;
    let source_offset = context.builder.build_int_add(
        arguments[1].into_int_value(),
        context.field_const(source_offset_shift as u64),
        "calldata_copy_source_offset",
    );
    let source = context.access_memory(
        source_offset,
        compiler_common::AddressSpace::Parent,
        "calldata_copy_source_pointer",
    );

    let size = arguments[2].into_int_value();

    let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyFromParent);
    context.build_call(
        intrinsic,
        &[
            destination.as_basic_value_enum(),
            source.as_basic_value_enum(),
            size.as_basic_value_enum(),
            context
                .integer_type(compiler_common::bitlength::BOOLEAN)
                .const_zero()
                .as_basic_value_enum(),
        ],
        "calldata_copy_memcpy_from_parent",
    );
    context.build_unconditional_branch(join_block);

    // TODO: remove if VM provides zeros after actual calldata
    context.set_basic_block(zero_block);
    let condition_block = context.append_basic_block("calldata_copy_zero_loop_condition");
    let body_block = context.append_basic_block("calldata_copy_zero_loop_body");
    let increment_block = context.append_basic_block("calldata_copy_zero_loop_increment");

    let index_pointer = context.build_alloca(
        context.field_type(),
        "calldata_copy_zero_loop_index_pointer",
    );
    context.build_store(index_pointer, arguments[0]);
    let range_end = context.builder.build_int_add(
        arguments[0].into_int_value(),
        arguments[2].into_int_value(),
        "calldata_copy_zero_loop_range_end",
    );
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(condition_block);
    let index_value = context
        .build_load(
            index_pointer,
            "calldata_copy_zero_loop_index_value_condition",
        )
        .into_int_value();
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        index_value,
        range_end,
        "calldata_copy_zero_loop_condition",
    );
    context.build_conditional_branch(condition, body_block, join_block);

    context.set_basic_block(increment_block);
    let index_value = context
        .build_load(
            index_pointer,
            "calldata_copy_zero_loop_index_value_increment",
        )
        .into_int_value();
    let incremented = context.builder.build_int_add(
        index_value,
        context.field_const(compiler_common::size::FIELD as u64),
        "calldata_copy_zero_loop_index_value_incremented",
    );
    context.build_store(index_pointer, incremented);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(body_block);
    let index_value = context
        .build_load(index_pointer, "calldata_copy_zero_loop_index_value_body")
        .into_int_value();
    let pointer = context.access_memory(
        index_value,
        compiler_common::AddressSpace::Heap,
        "calldata_copy_zero_pointer_body",
    );
    context.build_store(pointer, context.field_const(0));
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(join_block);
    None
}

///
/// Translates the calldata copy from the `codecopy` instruction.
///
pub fn codecopy<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let destination = context.access_memory(
        arguments[0].into_int_value(),
        compiler_common::AddressSpace::Heap,
        "calldata_codecopy_destination_pointer",
    );

    let source = context.access_memory(
        context.field_const(
            (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD) as u64,
        ),
        compiler_common::AddressSpace::Parent,
        "calldata_codecopy_source_pointer",
    );

    let size = arguments[2].into_int_value();

    let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyFromParent);
    context.build_call(
        intrinsic,
        &[
            destination.as_basic_value_enum(),
            source.as_basic_value_enum(),
            size.as_basic_value_enum(),
            context
                .integer_type(compiler_common::bitlength::BOOLEAN)
                .const_zero()
                .as_basic_value_enum(),
        ],
        "calldata_codecopy_memcpy_from_parent",
    );

    None
}
