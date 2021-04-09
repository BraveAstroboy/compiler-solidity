//!
//! The source code block.
//!

pub mod statement;

use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;

use self::statement::assignment::Assignment;
use self::statement::expression::Expression;
use self::statement::Statement;

///
/// The source code block.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    /// The block statements.
    pub statements: Vec<Statement>,
}

impl Block {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, mut initial: Option<Lexeme>) -> Self {
        let mut statements = Vec::new();

        loop {
            let lexeme = initial.take().unwrap_or_else(|| lexer.next());

            match lexeme {
                Lexeme::Keyword(_) => statements.push(Statement::parse(lexer, Some(lexeme))),
                Lexeme::Literal(_) => {
                    statements.push(Statement::Expression(Expression::parse(
                        lexer,
                        Some(lexeme),
                    )));
                }
                Lexeme::Identifier(_) => match lexer.peek() {
                    Lexeme::Symbol(Symbol::Assignment) => {
                        statements.push(Statement::Assignment(Assignment::parse(
                            lexer,
                            Some(lexeme),
                        )));
                    }
                    Lexeme::Symbol(Symbol::Comma) => {
                        statements.push(Statement::Assignment(Assignment::parse(
                            lexer,
                            Some(lexeme),
                        )));
                    }
                    _ => {
                        statements.push(Statement::Expression(Expression::parse(
                            lexer,
                            Some(lexeme),
                        )));
                    }
                },
                Lexeme::Symbol(Symbol::BracketCurlyLeft) => {
                    statements.push(Statement::Block(Block::parse(lexer, None)))
                }
                Lexeme::Symbol(Symbol::BracketCurlyRight) => break,
                _ => panic!("YUL is malformed"),
            }
        }

        Self { statements }
    }

    ///
    /// Translates a module block into LLVM.
    ///
    pub fn into_llvm_module(self, context: &mut LLVMContext) {
        for statement in self.statements.iter() {
            match statement {
                Statement::FunctionDefinition(statement) => {
                    statement.declare(context);
                }
                _ => unreachable!(),
            }
        }
        for statement in self.statements.into_iter() {
            match statement {
                Statement::FunctionDefinition(statement) => statement.into_llvm(context),
                _ => unreachable!(),
            }
        }
    }

    ///
    /// Translates a function or ordinar block into LLVM.
    ///
    pub fn into_llvm_local(self, context: &mut LLVMContext) {
        for statement in self.statements.into_iter() {
            match statement {
                // The scope can be cleaned up on exit, but let's LLVM do the job. We can also rely
                // on YUL renaming so we don't need to track scope.
                Statement::Block(block) => block.into_llvm_local(context),
                Statement::Expression(expression) => {
                    expression.into_llvm(context);
                }
                Statement::VariableDeclaration(statement) => statement.into_llvm(context),
                Statement::Assignment(statement) => statement.into_llvm(context),
                Statement::IfConditional(statement) => statement.into_llvm(context),
                Statement::Switch(statement) => statement.into_llvm(context),
                Statement::ForLoop(statement) => statement.into_llvm(context),
                Statement::Leave => {
                    context
                        .builder
                        .build_unconditional_branch(context.leave_block.expect("Always exists"));
                }
                Statement::Break => {
                    context
                        .builder
                        .build_unconditional_branch(context.break_block.expect("Always exists"));
                }
                Statement::Continue => {
                    context
                        .builder
                        .build_unconditional_branch(context.continue_block.expect("Always exists"));
                }
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::block::statement::Statement;
    use crate::parser::block::Block;
    use crate::parser::Module;

    #[test]
    fn ok_nested() {
        let input = r#"{
            {}
        }"#;

        let expected = Module {
            block: Block {
                statements: vec![Statement::Block(Block { statements: vec![] })],
            },
        };

        let result = crate::parse(input);
        assert_eq!(expected, result,);
    }

    #[test]
    #[should_panic]
    fn error_expected_bracket_curly_right() {
        let input = r#"{
            {}{}{{
        }"#;

        crate::parse(input);
    }
}
