//!
//! The if-conditional statement.
//!

use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;
use crate::yul::parser::statement::block::Block;
use crate::yul::parser::statement::expression::Expression;

///
/// The if-conditional statement.
///
#[derive(Debug, PartialEq, Clone)]
pub struct IfConditional {
    /// The condition expression.
    pub condition: Expression,
    /// The conditional block.
    pub block: Block,
}

impl IfConditional {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> anyhow::Result<Self> {
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

        let condition = Expression::parse(lexer, Some(lexeme))?;

        let block = Block::parse(lexer, None)?;

        Ok(Self { condition, block })
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for IfConditional
where
    D: compiler_llvm_context::Dependency,
{
    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        let condition = self
            .condition
            .into_llvm(context)?
            .expect("Always exists")
            .to_llvm()
            .into_int_value();
        let condition = context.builder().build_int_z_extend_or_bit_cast(
            condition,
            context.field_type(),
            "if_condition_extended",
        );
        let condition = context.builder().build_int_compare(
            inkwell::IntPredicate::NE,
            condition,
            context.field_const(0),
            "if_condition_compared",
        );
        let main_block = context.append_basic_block("if_main");
        let join_block = context.append_basic_block("if_join");
        context.build_conditional_branch(condition, main_block, join_block);
        context.set_basic_block(main_block);
        self.block.into_llvm(context)?;
        context.build_unconditional_branch(join_block);
        context.set_basic_block(join_block);

        Ok(())
    }
}
