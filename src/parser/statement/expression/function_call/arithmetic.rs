//!
//! Translates the arithmetic operations.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the arithmetic addition.
///
pub fn addition<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    Some(
        context
            .builder
            .build_int_add(
                arguments[0].into_int_value(),
                arguments[1].into_int_value(),
                "addition_result",
            )
            .as_basic_value_enum(),
    )
}

///
/// Translates the arithmetic subtraction.
///
pub fn subtraction<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    Some(
        context
            .builder
            .build_int_sub(
                arguments[0].into_int_value(),
                arguments[1].into_int_value(),
                "subtraction_result",
            )
            .as_basic_value_enum(),
    )
}

///
/// Translates the arithmetic multiplication.
///
pub fn multiplication<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    Some(
        context
            .builder
            .build_int_mul(
                arguments[0].into_int_value(),
                arguments[1].into_int_value(),
                "multiplication_result",
            )
            .as_basic_value_enum(),
    )
}

///
/// Translates the arithmetic division.
///
pub fn division<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let zero_block = context.append_basic_block("division_if_zero");
    let non_zero_block = context.append_basic_block("division_if_not_zero");
    let join_block = context.append_basic_block("division_if_join");

    let result_pointer = context.build_alloca(context.field_type(), "division_result_pointer");
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        arguments[1].into_int_value(),
        context.field_const(0),
        "division_condition",
    );
    context.build_conditional_branch(condition, zero_block, non_zero_block);

    context.set_basic_block(non_zero_block);
    let result = context.builder.build_int_unsigned_div(
        arguments[0].into_int_value(),
        arguments[1].into_int_value(),
        "division_result_non_zero",
    );
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0));
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "division_result");

    Some(result)
}

///
/// Translates the arithmetic remainder.
///
pub fn remainder<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let zero_block = context.append_basic_block("remainder_if_zero");
    let non_zero_block = context.append_basic_block("remainder_if_not_zero");
    let join_block = context.append_basic_block("remainder_if_join");

    let result_pointer = context.build_alloca(context.field_type(), "remainder_result_pointer");
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        arguments[1].into_int_value(),
        context.field_const(0),
        "remainder_if_zero_condition",
    );
    context.build_conditional_branch(condition, zero_block, non_zero_block);

    context.set_basic_block(non_zero_block);
    let result = context.builder.build_int_unsigned_rem(
        arguments[0].into_int_value(),
        arguments[1].into_int_value(),
        "remainder_result_non_zero",
    );
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0));
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "remainder_result");

    Some(result)
}

///
/// Translates the signed arithmetic division.
///
pub fn division_signed<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let zero_block = context.append_basic_block("division_if_zero");
    let non_zero_block = context.append_basic_block("division_if_not_zero");
    let join_block = context.append_basic_block("division_if_join");

    let result_pointer = context.build_alloca(context.field_type(), "division_result_pointer");
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        arguments[1].into_int_value(),
        context.field_const(0),
        "division_condition",
    );
    context.build_conditional_branch(condition, zero_block, non_zero_block);

    context.set_basic_block(non_zero_block);
    let result = context.builder.build_int_signed_div(
        arguments[0].into_int_value(),
        arguments[1].into_int_value(),
        "division_result_non_zero",
    );
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0));
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "division_result");

    Some(result)
}

///
/// Translates the signed arithmetic remainder.
///
pub fn remainder_signed<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let zero_block = context.append_basic_block("remainder_if_zero");
    let non_zero_block = context.append_basic_block("remainder_if_not_zero");
    let join_block = context.append_basic_block("remainder_if_join");

    let result_pointer = context.build_alloca(context.field_type(), "remainder_result_pointer");
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        arguments[1].into_int_value(),
        context.field_const(0),
        "remainder_if_zero_condition",
    );
    context.build_conditional_branch(condition, zero_block, non_zero_block);

    context.set_basic_block(non_zero_block);
    let result = context.builder.build_int_signed_rem(
        arguments[0].into_int_value(),
        arguments[1].into_int_value(),
        "remainder_result_non_zero",
    );
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0));
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "remainder_result");

    Some(result)
}
