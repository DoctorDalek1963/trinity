//! This module handles written expressions of matrices.
//!
//! We take an expression string, tokenise it with
//! [`tokenise_expression`](self::tokenise::tokenise_expression), turn it into an AST with
//! [`parse_tokens`](self::parser::parse_tokens) (see [`AstNode`](self::ast::AstNode)), and then
//! [`evaulate`](self::ast::AstNode::evaluate) it.

use thiserror::Error;

pub mod ast;
pub mod parser;
pub mod tokenise;

/// An error that occurred during tokenisation or during parsing.
#[derive(Debug, Error, PartialEq)]
pub enum TokeniseOrParseError<'i> {
    /// An error that occurred during tokenisation.
    #[error("{0}")]
    TokeniseError(self::tokenise::TokeniseError<'i>),

    /// An error that occurred during parsing.
    #[error("{0}")]
    ParseError(self::parser::ParseError<'i>),
}

// thiserror::Error has trouble deriving this with #[from]
impl<'i> From<self::tokenise::TokeniseError<'i>> for TokeniseOrParseError<'i> {
    fn from(value: self::tokenise::TokeniseError<'i>) -> Self {
        Self::TokeniseError(value)
    }
}

impl<'i> From<self::parser::ParseError<'i>> for TokeniseOrParseError<'i> {
    fn from(value: self::parser::ParseError<'i>) -> Self {
        Self::ParseError(value)
    }
}

/// Parse the expression directly from a string into an AST.
pub fn parse_expression(expression: &str) -> Result<self::ast::AstNode, TokeniseOrParseError> {
    let tokens = self::tokenise::tokenise_expression(expression)?;
    let ast = self::parser::parse_tokens(tokens)?;
    Ok(ast)
}
