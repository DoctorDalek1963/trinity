//! This module handles parsing a list of tokens into an AST.
//!
//! The grammar recognised by the parser is as follows:
//! ```text
//! expression        -> exponent ;
//! exponent          -> multiply ( "^" multiply )? ;
//! multiply          -> addition ( "*"? addition )* ;
//! addition          -> term ( ("+" | "-") term )* ;
//! term              -> matrixName | anonymousMatrix | rotationMatrix | NUMBER | "(" expression ")" ;
//! matrixName        -> See [`MatrixName`] struct
//! anonymousMatrix   -> anonymous2dMatrix | anonymous3dMatrix ;
//! anonymous2dMatrix -> "[" NUMBER NUMBER ";" NUMBER NUMBER "]" ;
//! anonymous3dMatrix -> "[" NUMBER NUMBER NUMBER ";" NUMBER NUMBER NUMBER ";" NUMBER NUMBER NUMBER "]" ;
//! rotationMatrix    -> "rot" "(" NUMBER ")" ;
//! ```

#[allow(dead_code, reason = "The implementation is still a WIP")]
mod nom_impl;
mod tokens;

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
