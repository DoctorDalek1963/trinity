//! This module implements functions for parsing [`TokenList`]s with [`nom`].

use super::tokens::TokenList;
use crate::matrix::expression::{ast::AstNode, tokenise::Token};
use nom::{branch::alt, bytes::complete::take, sequence::tuple, IResult, Parser};

/// Parse a single term of the AST. See [`crate::matrix::expression::parser`] for details on the
/// grammar.
fn parse_term<'n, 'l: 'n>(tokens: TokenList<'n, 'l>) -> IResult<TokenList<'n, 'l>, AstNode<'n>> {
    alt((
        parse_named_matrix,
        parse_rotation_matrix,
        parse_number,
        // parse_anonymous_2d_matrix,
        // parse_anonymous_3d_matrix,
        // tuple((
        //     tag(TokenList::new(&[Token::OpenParen])),
        //     parse_expression,
        //     tag(TokenList::new(&[Token::CloseParen])),
        // )),
    ))
    .parse(tokens)
}

/// Parse an [`AstNode::RotationMatrix`].
fn parse_rotation_matrix<'n, 'l: 'n>(
    tokens: TokenList<'n, 'l>,
) -> IResult<TokenList<'n, 'l>, AstNode<'n>> {
    tuple((
        consume_basic_token(Token::Rot),
        consume_basic_token(Token::OpenParen),
        parse_number,
        consume_basic_token(Token::CloseParen),
    ))
    .map(|(_, _, number, _)| match number {
        AstNode::Number(degrees) => AstNode::RotationMatrix { degrees },
        _ => panic!("parse_number should only ever return AstNode::Number, not {number:?}"),
    })
    .parse(tokens)
}

/// Consume a basic token that has no corresponding [`AstNode`].
///
/// I don't know why the restriction `'n: 'l` is necessary, but it won't compile without it.
fn consume_basic_token<'n: 'l, 'l: 'n>(
    expected_token: Token<'n>,
) -> impl Fn(TokenList<'n, 'l>) -> IResult<TokenList<'n, 'l>, ()> {
    move |tokens: TokenList<'n, 'l>| {
        let (rest, tok) = take(1usize)(tokens)?;
        if !tok.tokens.is_empty() && tok.tokens[0] == expected_token {
            Ok((rest, ()))
        } else {
            Err(nom::Err::Error(nom::error::Error::new(
                tokens,
                nom::error::ErrorKind::Tag,
            )))
        }
    }
}

/// Parse an [`AstNode::Number`].
fn parse_number<'n, 'l: 'n>(tokens: TokenList<'n, 'l>) -> IResult<TokenList<'n, 'l>, AstNode<'n>> {
    let (rest, tok) = take(1usize)(tokens)?;
    if tok.tokens.is_empty() {
        Err(nom::Err::Error(nom::error::Error::new(
            tokens,
            nom::error::ErrorKind::Tag,
        )))
    } else {
        match tok.tokens[0] {
            Token::Number(num) => Ok((rest, AstNode::Number(num))),
            _ => Err(nom::Err::Error(nom::error::Error::new(
                tokens,
                nom::error::ErrorKind::Tag,
            ))),
        }
    }
}

/// Parse an [`AstNode::NamedMatrix`].
fn parse_named_matrix<'n, 'l: 'n>(
    tokens: TokenList<'n, 'l>,
) -> IResult<TokenList<'n, 'l>, AstNode<'n>> {
    let (rest, tok) = take(1usize)(tokens)?;
    if tok.tokens.is_empty() {
        Err(nom::Err::Error(nom::error::Error::new(
            tokens,
            nom::error::ErrorKind::Tag,
        )))
    } else {
        match tok.tokens[0] {
            Token::NamedMatrix(matrix_name) => Ok((rest, AstNode::NamedMatrix(matrix_name))),
            _ => Err(nom::Err::Error(nom::error::Error::new(
                tokens,
                nom::error::ErrorKind::Tag,
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::MatrixName;
    use Token as T;
    use TokenList as TL;

    #[test]
    fn parse_simple_success() {
        assert_eq!(
            parse_named_matrix(TL::new(&[T::NamedMatrix(MatrixName::new("M"))])),
            Ok((TL::new(&[]), AstNode::NamedMatrix(MatrixName::new("M"))))
        );

        assert_eq!(
            parse_number(TL::new(&[T::Number(12.5)])),
            Ok((TL::new(&[]), AstNode::Number(12.5)))
        );

        assert_eq!(
            parse_rotation_matrix(TL::new(&[
                T::Rot,
                T::OpenParen,
                T::Number(45.),
                T::CloseParen
            ])),
            Ok((TL::new(&[]), AstNode::RotationMatrix { degrees: 45. }))
        );
    }
}
