//! This module handles parsing a list of tokens into an AST.
//!
//! The grammar recognised by the parser is as follows:
//! ```text
//! expression        -> addition ;
//! addition          -> multiply ( ("+" | "-") multiply )* ;
//! multiply          -> divide ( "*" divide )* ;
//! divide            -> exponent ( "/" exponent )* ;
//! exponent          -> term ( "^" term )? ;
//! term              -> "-"? term | matrixName | anonymousMatrix | rotationMatrix | NUMBER | "(" expression ")" ;
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
use thiserror::Error;

/// The default error used by [`nom::IResult`].
type NomError = ::nom::Err<::nom::error::Error<Vec<Token>>>;

/// An error that occurred during parsing.
#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    /// An error created by [`nom`].
    #[error("Internal nom error: {0:?}")]
    NomError(#[from] NomError),

    /// Some of the input was left unparsed.
    #[error("Unconsumed input after tokenising expression: '{0:?}'")]
    UnconsumedInput(Vec<Token>),
}

/// Parse a list of tokens into an AST.
pub fn parse_tokens_into_ast<'l>(tokens: &'l [Token]) -> Result<AstNode, ParseError> {
    let (token_list, ast) = self::nom_impl::parse_expression(self::tokens::TokenList::new(&tokens))
        .map_err(|err| err.map_input(|token_list| token_list.tokens.to_vec()))?;

    if !token_list.tokens.is_empty() {
        return Err(ParseError::UnconsumedInput(token_list.tokens.to_vec()));
    }

    Ok(ast)
}
