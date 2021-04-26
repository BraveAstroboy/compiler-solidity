//!
//! The YUL code.
//!

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;
use crate::parser::statement::block::Block;
use crate::parser::statement::object::Object;

///
/// The YUL code.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Code {
    /// The main block.
    pub block: Block,
}

impl Code {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        match lexeme {
            Lexeme::Keyword(Keyword::Code) => {}
            lexeme => return Err(ParserError::expected_one_of(vec!["code"], lexeme, None).into()),
        }

        let block = Block::parse(lexer, None)?;

        Ok(Self { block })
    }

    ///
    /// Converts the element into a test object.
    ///
    pub fn into_test_object(self) -> Object {
        Object {
            identifier: "Test".to_owned(),
            code: self,
            object: None,
        }
    }
}

impl ILLVMWritable for Code {
    fn into_llvm(self, context: &mut LLVMContext) {
        self.block.into_llvm_code(context);
    }
}
