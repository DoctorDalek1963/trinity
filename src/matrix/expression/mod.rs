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

// This currently doesn't work because the compile says that `ast` references `tokens` even thought
// it should only be referencing `expression`. But I think I'm gonna have to refactor `MatrixName`
// to use proper string interning anyway (where would all those referenes actually be pointing in
// the finished program anyway? I need some kind of interning pool), then I can strip out most of
// this lifetime faffery.

// /// Parse the expression directly from a string into an AST.
// pub fn parse_expression<'n>(
//     expression: &'n str,
// ) -> Result<self::ast::AstNode<'n>, TokeniseOrParseError<'n>> {
//     let tokens = self::tokenise::tokenise_expression(expression)?;
//     let ast = self::parser::parse_tokens_into_ast(&tokens)?;
//     Ok(ast)
// }
