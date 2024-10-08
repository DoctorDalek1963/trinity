//! This module handles tokenising a matrix expression string into a list of [`Token`]s.

use crate::matrix::{MatrixName, LEADING_MATRIX_NAME_REGEX};
use nom::{
    branch::alt, bytes::complete::tag, character::complete::multispace1, multi::many1,
    number::complete::float, IResult, Parser,
};
use nom_regex::str::re_find;
use thiserror::Error;

/// A single token in the token list that results from tokenisation.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// A named matrix. See [`MatrixName`].
    NamedMatrix(MatrixName),

    /// A numeric literal.
    Number(f64),

    /// The rotation command `rot`.
    Rot,

    /// The `+` symbol.
    Plus,

    /// The `-` symbol.
    Minus,

    /// The `*` symbol.
    Star,

    /// The `/` symbol.
    Slash,

    /// The `^` symbol.
    Caret,

    /// The `;` symbol.
    Semicolon,

    /// The `(` symbol.
    OpenParen,

    /// The `)` symbol.
    CloseParen,

    /// The `[` symbol.
    OpenSquareBracket,

    /// The `]` symbol.
    CloseSquareBracket,

    /// The `{` symbol.
    OpenBrace,

    /// The `}` symbol.
    CloseBrace,
}

/// The default error used by [`nom::IResult`].
type NomError<'i> = ::nom::Err<::nom::error::Error<&'i str>>;

/// An error that occurred during tokenisation.
#[derive(Debug, Error, PartialEq)]
pub enum TokeniseError<'i> {
    /// An error created by [`nom`].
    #[error("Internal nom error: {nom_error:?}")]
    NomError {
        /// The internal error from [`nom`].
        nom_error: NomError<'i>,
    },

    /// Some of the input was left un-tokenised.
    #[error("Unconsumed input after tokenising expression: '{0}'")]
    UnconsumedInput(&'i str),
}

impl<'i> From<NomError<'i>> for TokeniseError<'i> {
    fn from(nom_error: NomError<'i>) -> Self {
        TokeniseError::NomError { nom_error }
    }
}

/// Tokenise the whole expression into a list of tokens.
///
/// Note that the tokeniser cannot tokenise negative numbers. It will instead tokenise the minus
/// sign and then tokenise the positive number.
///
/// ```
/// # use trinity::matrix::expression::tokenise::{Token, tokenise_expression};
/// assert_eq!(
///     tokenise_expression("-1"),
///     Ok(vec![Token::Minus, Token::Number(1.0)])
/// );
/// assert_eq!(
///     tokenise_expression("5-3"),
///     Ok(vec![Token::Number(5.0), Token::Minus, Token::Number(3.0)])
/// );
/// assert_eq!(
///     tokenise_expression("5+(-3)"),
///     Ok(vec![
///         Token::Number(5.0),
///         Token::Plus,
///         Token::OpenParen,
///         Token::Minus,
///         Token::Number(3.0),
///         Token::CloseParen
///     ])
/// );
/// ```
#[allow(clippy::needless_lifetimes)]
pub fn tokenise_expression<'i>(expression: &'i str) -> Result<Vec<Token>, TokeniseError<'i>> {
    let (input, opt_tokens) = many1(alt((
        tokenise_named_matrix.map(Some),
        tokenise_rot.map(Some),
        tokenise_punctuation.map(Some),
        tokenise_number.map(Some),
        multispace1.map(|_| None),
    )))(expression)?;

    if !input.is_empty() {
        return Err(TokeniseError::UnconsumedInput(input));
    }

    Ok(opt_tokens.into_iter().flatten().collect())
}

/// Tokenise a single named matrix from the expression.
fn tokenise_named_matrix(input: &str) -> IResult<&str, Token> {
    re_find(LEADING_MATRIX_NAME_REGEX.clone())
        .map(|name| Token::NamedMatrix(MatrixName::new(name)))
        .parse(input)
}

/// Tokenise a single number from the expression.
fn tokenise_number(input: &str) -> IResult<&str, Token> {
    float.map(|num| Token::Number(num as f64)).parse(input)
}

/// Tokenise a rotation command from the expression.
fn tokenise_rot(input: &str) -> IResult<&str, Token> {
    tag("rot").map(|_| Token::Rot).parse(input)
}

/// Tokenise a piece of punctuation from the expression.
fn tokenise_punctuation(input: &str) -> IResult<&str, Token> {
    alt((
        tag("+").map(|_| Token::Plus),
        tag("-").map(|_| Token::Minus),
        tag("*").map(|_| Token::Star),
        tag("/").map(|_| Token::Slash),
        tag("^").map(|_| Token::Caret),
        tag(";").map(|_| Token::Semicolon),
        tag("(").map(|_| Token::OpenParen),
        tag(")").map(|_| Token::CloseParen),
        tag("[").map(|_| Token::OpenSquareBracket),
        tag("]").map(|_| Token::CloseSquareBracket),
        tag("{").map(|_| Token::OpenBrace),
        tag("}").map(|_| Token::CloseBrace),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenise_named_matrix() {
        let valid_names = [
            "M",
            "Mat",
            "A_",
            "X_y",
            "Dave",
            "N",
            "T",
            "Some_really_long_matrix_name_but_its_okay_because_it_fits_the_rules",
            "Abc",
        ];
        for name in valid_names {
            assert_eq!(
                tokenise_named_matrix(name),
                Ok(("", Token::NamedMatrix(MatrixName::new(name)))),
                "'{name}' should be valid"
            );
        }

        assert_eq!(
            tokenise_named_matrix("ABC"),
            Ok(("BC", Token::NamedMatrix(MatrixName::new("A"))))
        );

        assert_eq!(
            tokenise_named_matrix("M * 2"),
            Ok((" * 2", Token::NamedMatrix(MatrixName::new("M"))))
        );

        assert_eq!(
            tokenise_named_matrix("Z-2"),
            Ok(("-2", Token::NamedMatrix(MatrixName::new("Z"))))
        );

        assert_eq!(
            tokenise_named_matrix("X:C"),
            Ok((":C", Token::NamedMatrix(MatrixName::new("X"))))
        );

        assert_eq!(
            tokenise_named_matrix("Name with spaces"),
            Ok((" with spaces", Token::NamedMatrix(MatrixName::new("Name"))))
        );

        assert_eq!(
            tokenise_named_matrix("WhatAboutPunctuation?"),
            Ok((
                "AboutPunctuation?",
                Token::NamedMatrix(MatrixName::new("What"))
            ))
        );

        assert_eq!(
            tokenise_named_matrix("It's"),
            Ok(("'s", Token::NamedMatrix(MatrixName::new("It"))))
        );

        let invalid_names = ["", "m", " M", "x", "my_matrix", "::"];
        for name in invalid_names {
            assert!(
                tokenise_named_matrix(name).is_err(),
                "'{name}' should be invalid"
            );
        }
    }

    #[test]
    fn tokenise_expression_success() {
        use super::Token as T;

        assert_eq!(
            tokenise_expression("M^2 * [1 2; 3 -5]"),
            Ok(vec![
                T::NamedMatrix(MatrixName::new("M")),
                T::Caret,
                T::Number(2.),
                T::Star,
                T::OpenSquareBracket,
                T::Number(1.),
                T::Number(2.),
                T::Semicolon,
                T::Number(3.),
                T::Minus,
                T::Number(5.),
                T::CloseSquareBracket
            ])
        );

        assert_eq!(
            tokenise_expression("[1;23]^{2*(3+9)}-6"),
            Ok(vec![
                T::OpenSquareBracket,
                T::Number(1.),
                T::Semicolon,
                T::Number(23.),
                T::CloseSquareBracket,
                T::Caret,
                T::OpenBrace,
                T::Number(2.),
                T::Star,
                T::OpenParen,
                T::Number(3.),
                T::Plus,
                T::Number(9.),
                T::CloseParen,
                T::CloseBrace,
                T::Minus,
                T::Number(6.)
            ])
        );

        assert_eq!(
            tokenise_expression("M ^ {-1}"),
            Ok(vec![
                T::NamedMatrix(MatrixName::new("M")),
                T::Caret,
                T::OpenBrace,
                T::Minus,
                T::Number(1.),
                T::CloseBrace,
            ])
        );

        assert_eq!(
            tokenise_expression("M^-1+X"),
            Ok(vec![
                T::NamedMatrix(MatrixName::new("M")),
                T::Caret,
                T::Minus,
                T::Number(1.),
                T::Plus,
                T::NamedMatrix(MatrixName::new("X")),
            ])
        );

        assert_eq!(
            tokenise_expression("rot(45) * ((1 + 2) * My_matrix)"),
            Ok(vec![
                T::Rot,
                T::OpenParen,
                T::Number(45.),
                T::CloseParen,
                T::Star,
                T::OpenParen,
                T::OpenParen,
                T::Number(1.),
                T::Plus,
                T::Number(2.),
                T::CloseParen,
                T::Star,
                T::NamedMatrix(MatrixName::new("My_matrix")),
                T::CloseParen,
            ])
        );

        assert_eq!(
            tokenise_expression("ABC + A2B"),
            Ok(vec![
                T::NamedMatrix(MatrixName::new("A")),
                T::NamedMatrix(MatrixName::new("B")),
                T::NamedMatrix(MatrixName::new("C")),
                T::Plus,
                T::NamedMatrix(MatrixName::new("A")),
                T::Number(2.),
                T::NamedMatrix(MatrixName::new("B")),
            ])
        );
    }

    #[test]
    fn tokenise_expression_abc() {
        assert_eq!(
            tokenise_expression("ABC"),
            Ok(vec![
                Token::NamedMatrix(MatrixName::new("A")),
                Token::NamedMatrix(MatrixName::new("B")),
                Token::NamedMatrix(MatrixName::new("C"))
            ])
        );

        assert_eq!(
            tokenise_expression("ABc"),
            Ok(vec![
                Token::NamedMatrix(MatrixName::new("A")),
                Token::NamedMatrix(MatrixName::new("Bc")),
            ])
        );

        assert_eq!(
            tokenise_expression("AbC"),
            Ok(vec![
                Token::NamedMatrix(MatrixName::new("Ab")),
                Token::NamedMatrix(MatrixName::new("C"))
            ])
        );

        assert_eq!(
            tokenise_expression("Abc"),
            Ok(vec![Token::NamedMatrix(MatrixName::new("Abc"))])
        );

        assert_eq!(
            tokenise_expression("aBC"),
            Err(TokeniseError::NomError {
                nom_error: nom::Err::Error(nom::error::Error::new(
                    "aBC",
                    nom::error::ErrorKind::MultiSpace
                ))
            })
        );

        assert_eq!(
            tokenise_expression("aBc"),
            Err(TokeniseError::NomError {
                nom_error: nom::Err::Error(nom::error::Error::new(
                    "aBc",
                    nom::error::ErrorKind::MultiSpace
                ))
            })
        );

        assert_eq!(
            tokenise_expression("abC"),
            Err(TokeniseError::NomError {
                nom_error: nom::Err::Error(nom::error::Error::new(
                    "abC",
                    nom::error::ErrorKind::MultiSpace
                ))
            })
        );

        assert_eq!(
            tokenise_expression("abc"),
            Err(TokeniseError::NomError {
                nom_error: nom::Err::Error(nom::error::Error::new(
                    "abc",
                    nom::error::ErrorKind::MultiSpace
                ))
            })
        );
    }

    #[test]
    fn tokenise_expression_failure() {
        assert_eq!(
            tokenise_expression("@"),
            Err(TokeniseError::NomError {
                nom_error: ::nom::Err::Error(::nom::error::Error {
                    input: "@",
                    code: ::nom::error::ErrorKind::MultiSpace
                })
            })
        );

        assert_eq!(
            tokenise_expression(" []@"),
            Err(TokeniseError::UnconsumedInput("@"))
        );

        assert_eq!(
            tokenise_expression(std::str::from_utf8(&[10, 5, 91]).unwrap()),
            Err(TokeniseError::UnconsumedInput(
                std::str::from_utf8(&[5, 91]).unwrap()
            ))
        );

        assert_eq!(
            tokenise_expression("word"),
            Err(TokeniseError::NomError {
                nom_error: ::nom::Err::Error(::nom::error::Error {
                    input: "word",
                    code: ::nom::error::ErrorKind::MultiSpace
                })
            })
        );
    }
}
