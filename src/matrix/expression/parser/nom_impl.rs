//! This module implements functions for parsing [`TokenList`]s with [`nom`].

use super::tokens::TokenList;
use crate::matrix::expression::{ast::AstNode, tokenise::Token};
use glam::{DMat2, DMat3, DVec2, DVec3};
use nom::{branch::alt, bytes::complete::take, sequence::tuple, IResult, Parser};

/// Parse a single term of the AST. See [`crate::matrix::expression::parser`] for details on the
/// grammar.
fn parse_term<'n, 'l: 'n>(tokens: TokenList<'n, 'l>) -> IResult<TokenList<'n, 'l>, AstNode<'n>> {
    alt((
        parse_named_matrix,
        parse_rotation_matrix,
        parse_number,
        parse_anonymous_2d_matrix,
        parse_anonymous_3d_matrix,
        // tuple((
        //     consume_basic_token(Token::OpenParen),
        //     parse_expression,
        //     consume_basic_token(Token::CloseParen),
        // ))
        // .map(|((), expression, ())| expression),
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

/// Parse an anonymous 2D matrix, like `[1 2; 3 4]`.
fn parse_anonymous_2d_matrix<'n, 'l: 'n>(
    tokens: TokenList<'n, 'l>,
) -> IResult<TokenList<'n, 'l>, AstNode<'n>> {
    let (tokens, ()) = consume_basic_token(Token::OpenSquareBracket)(tokens)?;
    let (tokens, ix) = parse_number(tokens)?;
    let (tokens, jx) = parse_number(tokens)?;
    let (tokens, ()) = consume_basic_token(Token::Semicolon)(tokens)?;
    let (tokens, iy) = parse_number(tokens)?;
    let (tokens, jy) = parse_number(tokens)?;
    let (tokens, ()) = consume_basic_token(Token::CloseSquareBracket)(tokens)?;

    let matrix = match (ix, jx, iy, jy) {
        (AstNode::Number(ix), AstNode::Number(jx), AstNode::Number(iy), AstNode::Number(jy)) => {
            AstNode::Anonymous2dMatrix(DMat2::from_cols(DVec2::new(ix, iy), DVec2::new(jx, jy)))
        }
        _ => panic!("parse_number should only ever return AstNode::Number"),
    };

    Ok((tokens, matrix))
}

/// Parse an anonymous 3D matrix, like `[1 2 3; 4 5 6; 7 8 9]`.
fn parse_anonymous_3d_matrix<'n, 'l: 'n>(
    tokens: TokenList<'n, 'l>,
) -> IResult<TokenList<'n, 'l>, AstNode<'n>> {
    let (tokens, ()) = consume_basic_token(Token::OpenSquareBracket)(tokens)?;
    let (tokens, ix) = parse_number(tokens)?;
    let (tokens, jx) = parse_number(tokens)?;
    let (tokens, kx) = parse_number(tokens)?;
    let (tokens, ()) = consume_basic_token(Token::Semicolon)(tokens)?;
    let (tokens, iy) = parse_number(tokens)?;
    let (tokens, jy) = parse_number(tokens)?;
    let (tokens, ky) = parse_number(tokens)?;
    let (tokens, ()) = consume_basic_token(Token::Semicolon)(tokens)?;
    let (tokens, iz) = parse_number(tokens)?;
    let (tokens, jz) = parse_number(tokens)?;
    let (tokens, kz) = parse_number(tokens)?;
    let (tokens, ()) = consume_basic_token(Token::CloseSquareBracket)(tokens)?;

    let matrix = match (ix, jx, kx, iy, jy, ky, iz, jz, kz) {
        (
            AstNode::Number(ix),
            AstNode::Number(jx),
            AstNode::Number(kx),
            AstNode::Number(iy),
            AstNode::Number(jy),
            AstNode::Number(ky),
            AstNode::Number(iz),
            AstNode::Number(jz),
            AstNode::Number(kz),
        ) => AstNode::Anonymous3dMatrix(DMat3::from_cols(
            DVec3::new(ix, iy, iz),
            DVec3::new(jx, jy, jz),
            DVec3::new(kx, ky, kz),
        )),
        _ => panic!("parse_number should only ever return AstNode::Number"),
    };

    Ok((tokens, matrix))
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
            Ok((TL::EMPTY, AstNode::NamedMatrix(MatrixName::new("M"))))
        );

        assert_eq!(
            parse_number(TL::new(&[T::Number(12.5)])),
            Ok((TL::EMPTY, AstNode::Number(12.5)))
        );

        assert_eq!(
            parse_rotation_matrix(TL::new(&[
                T::Rot,
                T::OpenParen,
                T::Number(45.),
                T::CloseParen
            ])),
            Ok((TL::EMPTY, AstNode::RotationMatrix { degrees: 45. }))
        );

        assert_eq!(
            parse_anonymous_2d_matrix(TL::new(&[
                T::OpenSquareBracket,
                T::Number(1.),
                T::Number(2.),
                T::Semicolon,
                T::Number(3.),
                T::Number(4.),
                T::CloseSquareBracket,
            ])),
            Ok((
                TL::EMPTY,
                AstNode::Anonymous2dMatrix(DMat2::from_cols(
                    DVec2::new(1., 3.),
                    DVec2::new(2., 4.)
                ))
            ))
        );

        assert_eq!(
            parse_anonymous_3d_matrix(TL::new(&[
                T::OpenSquareBracket,
                T::Number(1.),
                T::Number(2.),
                T::Number(3.),
                T::Semicolon,
                T::Number(4.),
                T::Number(5.),
                T::Number(6.),
                T::Semicolon,
                T::Number(7.),
                T::Number(8.),
                T::Number(9.),
                T::CloseSquareBracket,
            ])),
            Ok((
                TL::EMPTY,
                AstNode::Anonymous3dMatrix(DMat3::from_cols(
                    DVec3::new(1., 4., 7.),
                    DVec3::new(2., 5., 8.),
                    DVec3::new(3., 6., 9.),
                ))
            ))
        );
    }
}
