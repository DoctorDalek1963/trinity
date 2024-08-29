//! This module handles parsing streams of tokens.

use super::{ast::AstNode, tokenise::Token};
use std::fmt;
use thiserror::Error;

/// An error that occurred during parsing.
#[derive(Clone, Copy, Debug, Error, PartialEq)]
pub struct ParseError<'i> {
    /// The token that caused the error.
    token: Token<'i>,

    /// The index of the offending token in the provided token list.
    index: usize,
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

/// Parse a list of tokens into an AST.
pub fn parse_tokens(_tokens: Vec<Token>) -> Result<AstNode, ParseError> {
    todo!("Actually parse tokens")
}
