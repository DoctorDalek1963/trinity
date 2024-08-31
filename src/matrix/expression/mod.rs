//! This module handles written expressions of matrices.
//!
//! We take an expression string, tokenise it with
//! [`tokenise_expression`](self::tokenise::tokenise_expression), turn it into an AST with
//! [`parse_tokens_into_ast`](self::parser::parse_tokens_into_ast) (see
//! [`AstNode`](self::ast::AstNode)), and then [`evaulate`](self::ast::AstNode::evaluate) it.

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
    ParseError(#[from] self::parser::ParseError),
}

// thiserror::Error has trouble deriving this with #[from]
impl<'i> From<self::tokenise::TokeniseError<'i>> for TokeniseOrParseError<'i> {
    fn from(value: self::tokenise::TokeniseError<'i>) -> Self {
        Self::TokeniseError(value)
    }
}

/// Parse the expression directly from a string into an AST.
pub fn parse_expression_from_string(
    expression: &str,
) -> Result<self::ast::AstNode, TokeniseOrParseError> {
    let tokens = self::tokenise::tokenise_expression(expression)?;
    let ast = self::parser::parse_tokens_into_ast(&tokens)?;
    Ok(ast)
}
