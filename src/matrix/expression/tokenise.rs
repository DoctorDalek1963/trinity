//! This module handles tokenising a matrix expression string into a list of [`Token`]s.

use crate::matrix::MatrixName;
use lazy_static::lazy_static;
use nom::{
    branch::alt, bytes::complete::tag, character::complete::multispace1, multi::many1,
    number::complete::float, IResult, Parser,
};
use nom_regex::str::re_find;
use regex::Regex;

lazy_static! {
    /// The regular expression used to validate matrix names during tokenisation.
    /// See [`MatrixName`].
    pub static ref MATRIX_NAME_REGEX: Regex = Regex::new(r"^[A-Z][A-Za-z0-9_]*").unwrap();
}

/// A single token in the token list that results from tokenisation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Token<'n> {
    /// A named matrix. See [`MatrixName`].
    NamedMatrix(MatrixName<'n>),

    /// A numeric literal.
    Number(f64),

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
#[derive(Debug, PartialEq)]
pub struct TokeniseError<'n> {
    /// The internal error from [`nom`].
    nom_error: NomError<'n>,
}

impl<'n> From<NomError<'n>> for TokeniseError<'n> {
    fn from(nom_error: NomError<'n>) -> Self {
        TokeniseError { nom_error }
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
pub fn tokenise_expression<'n>(expression: &'n str) -> Result<Vec<Token<'n>>, TokeniseError<'n>> {
    let (input, opt_tokens) = many1(alt((
        tokenise_named_matrix.map(Some),
        tokenise_punctuation.map(Some),
        tokenise_number.map(Some),
        multispace1.map(|_| None),
    )))(expression)?;

    debug_assert!(
        input.is_empty(),
        "The unparsed input should be empty, not '{input}'"
    );

    Ok(opt_tokens.into_iter().flatten().collect())
}

/// Tokenise a single named matrix from the expression.
fn tokenise_named_matrix(input: &str) -> IResult<&str, Token> {
    re_find(MATRIX_NAME_REGEX.clone())
        .map(|name| {
            debug_assert!(!name.is_empty() && name.starts_with(|c: char| c.is_uppercase()));
            Token::NamedMatrix(MatrixName(name))
        })
        .parse(input)
}

/// Tokenise a single number from the expression.
fn tokenise_number(input: &str) -> IResult<&str, Token> {
    float.map(|num| Token::Number(num as f64)).parse(input)
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
        let valid_names = ["M", "Mat", "A2", "X_Y3", "Dave", "N", "T"];
        for name in valid_names {
            assert_eq!(
                tokenise_named_matrix(name),
                Ok(("", Token::NamedMatrix(MatrixName(name)))),
                "'{name}' should be valid"
            );
        }

        assert_eq!(
            tokenise_named_matrix("M * 2"),
            Ok((" * 2", Token::NamedMatrix(MatrixName("M"))))
        );

        assert_eq!(
            tokenise_named_matrix("Z-2"),
            Ok(("-2", Token::NamedMatrix(MatrixName("Z"))))
        );

        assert_eq!(
            tokenise_named_matrix("X:C"),
            Ok((":C", Token::NamedMatrix(MatrixName("X"))))
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
    fn test_tokenise_expression() {
        use super::Token as T;

        assert_eq!(
            tokenise_expression("M^2 * [1 2; 3 -5]"),
            Ok(vec![
                T::NamedMatrix(MatrixName("M")),
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
    }
}
