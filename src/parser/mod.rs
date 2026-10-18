pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod validator;

pub use ast::*;
pub use lexer::Lexer;
pub use parser::Parser;
pub use token::{Span, Token, TokenKind};
pub use validator::Validator;

use crate::error::Result;

pub fn parse_and_validate(source: &str) -> Result<Schema> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let schema = parser.parse_schema()?;
    let validator = Validator::new(&schema);
    validator.validate()?;
    Ok(schema)
}
