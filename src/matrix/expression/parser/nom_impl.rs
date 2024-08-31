//! This module implements functions for parsing [`TokenList`]s with [`nom`].

use super::tokens::TokenList;
use crate::matrix::expression::{ast::AstNode, tokenise::Token};
use glam::{DMat2, DMat3, DVec2, DVec3};
use nom::{branch::alt, bytes::complete::take, sequence::tuple, IResult, Parser};

/// Parse a matrix expression from a list of tokens.
pub fn parse_expression(tokens: TokenList) -> IResult<TokenList, AstNode> {
    // alt((
    //     parse_exponent,
    //     parse_divide,
    //     parse_multiply,
    //     parse_addition,
    //     parse_term,
    // ))
    // .parse(tokens)
    parse_addition(tokens)
}

/// Parse an addition.
fn parse_addition(tokens: TokenList) -> IResult<TokenList, AstNode> {
    let (tokens, left) = parse_multiply(tokens)?;

    match consume_basic_token(Token::Plus)(tokens) {
        Ok((tokens, ())) => {
            let (tokens, right) = parse_addition(tokens)?;

            Ok((
                tokens,
                AstNode::Add {
                    left: Box::new(left),
                    right: Box::new(right),
                },
            ))
        }
        Err(_) => Ok((tokens, left)),
    }
}

/// Parse a multiplication.
fn parse_multiply(tokens: TokenList) -> IResult<TokenList, AstNode> {
    let (tokens, left) = parse_divide(tokens)?;

    match consume_basic_token(Token::Star)(tokens) {
        Ok((tokens, ())) => {
            let (tokens, right) = parse_multiply(tokens)?;

            Ok((
                tokens,
                AstNode::Multiply {
                    left: Box::new(left),
                    right: Box::new(right),
                },
            ))
        }
        Err(_) => Ok((tokens, left)),
    }
}

/// Parse a division.
fn parse_divide(tokens: TokenList) -> IResult<TokenList, AstNode> {
    let (tokens, left) = parse_exponent(tokens)?;

    match consume_basic_token(Token::Slash)(tokens) {
        Ok((tokens, ())) => {
            let (tokens, right) = parse_divide(tokens)?;

            Ok((
                tokens,
                AstNode::Divide {
                    left: Box::new(left),
                    right: Box::new(right),
                },
            ))
        }
        Err(_) => Ok((tokens, left)),
    }
}

/// Parse an exponentiation.
fn parse_exponent(tokens: TokenList) -> IResult<TokenList, AstNode> {
    let (tokens, base) = parse_term(tokens)?;

    match consume_basic_token(Token::Caret)(tokens) {
        Ok((tokens, ())) => {
            let (tokens, power) = {
                match consume_basic_token(Token::OpenBrace)(tokens) {
                    Ok((tokens, ())) => {
                        let (tokens, power) = parse_exponent(tokens)?;
                        let (tokens, ()) = consume_basic_token(Token::CloseBrace)(tokens)?;
                        (tokens, power)
                    }
                    Err(_) => parse_exponent(tokens)?,
                }
            };

            Ok((
                tokens,
                AstNode::Exponent {
                    base: Box::new(base),
                    power: Box::new(power),
                },
            ))
        }
        Err(_) => Ok((tokens, base)),
    }
}

/// Parse a single term of the AST. See [`crate::matrix::expression::parser`] for details on the
/// grammar.
fn parse_term(tokens: TokenList) -> IResult<TokenList, AstNode> {
    alt((
        tuple((consume_basic_token(Token::Minus), parse_term))
            .map(|((), term)| AstNode::Negate(Box::new(term))),
        parse_named_matrix,
        parse_rotation_matrix,
        parse_number,
        parse_anonymous_2d_matrix,
        parse_anonymous_3d_matrix,
        tuple((
            consume_basic_token(Token::OpenParen),
            parse_expression,
            consume_basic_token(Token::CloseParen),
        ))
        .map(|((), expression, ())| expression),
    ))
    .parse(tokens)
}

/// Parse an [`AstNode::RotationMatrix`].
fn parse_rotation_matrix(tokens: TokenList) -> IResult<TokenList, AstNode> {
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
fn parse_anonymous_2d_matrix(tokens: TokenList) -> IResult<TokenList, AstNode> {
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
fn parse_anonymous_3d_matrix(tokens: TokenList) -> IResult<TokenList, AstNode> {
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
fn consume_basic_token<'l>(
    expected_token: Token,
) -> impl Fn(TokenList<'l>) -> IResult<TokenList<'l>, ()> {
    move |tokens: TokenList<'l>| {
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
fn parse_number(tokens: TokenList) -> IResult<TokenList, AstNode> {
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
fn parse_named_matrix(tokens: TokenList) -> IResult<TokenList, AstNode> {
    let (rest, tok) = take(1usize)(tokens)?;
    if tok.tokens.is_empty() {
        Err(nom::Err::Error(nom::error::Error::new(
            tokens,
            nom::error::ErrorKind::Tag,
        )))
    } else {
        match &tok.tokens[0] {
            Token::NamedMatrix(matrix_name) => {
                Ok((rest, AstNode::NamedMatrix(matrix_name.clone())))
            }
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

        assert_eq!(
            parse_exponent(TL::new(&[
                T::NamedMatrix(MatrixName::new("M")),
                T::Caret,
                T::Number(2.)
            ])),
            Ok((
                TL::EMPTY,
                AstNode::Exponent {
                    base: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                    power: Box::new(AstNode::Number(2.))
                }
            ))
        );

        assert_eq!(
            parse_exponent(TL::new(&[
                T::NamedMatrix(MatrixName::new("M")),
                T::Caret,
                T::Minus,
                T::Number(1.)
            ])),
            Ok((
                TL::EMPTY,
                AstNode::Exponent {
                    base: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                    power: Box::new(AstNode::Negate(Box::new(AstNode::Number(1.))))
                }
            ))
        );

        assert_eq!(
            parse_exponent(TL::new(&[
                T::NamedMatrix(MatrixName::new("M")),
                T::Caret,
                T::OpenBrace,
                T::Minus,
                T::Number(2.5),
                T::CloseBrace,
            ])),
            Ok((
                TL::EMPTY,
                AstNode::Exponent {
                    base: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                    power: Box::new(AstNode::Negate(Box::new(AstNode::Number(2.5))))
                }
            ))
        );

        assert_eq!(
            parse_exponent(TL::new(&[
                T::NamedMatrix(MatrixName::new("M")),
                T::Caret,
                T::OpenBrace,
                T::Number(0.5),
                T::CloseBrace,
            ])),
            Ok((
                TL::EMPTY,
                AstNode::Exponent {
                    base: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                    power: Box::new(AstNode::Number(0.5))
                }
            ))
        );

        assert_eq!(
            parse_divide(TL::new(&[T::Number(2.), T::Slash, T::Number(3.),])),
            Ok((
                TL::EMPTY,
                AstNode::Divide {
                    left: Box::new(AstNode::Number(2.)),
                    right: Box::new(AstNode::Number(3.))
                }
            ))
        );

        assert_eq!(
            parse_multiply(TL::new(&[
                T::Number(2.),
                T::Star,
                T::NamedMatrix(MatrixName::new("M")),
            ])),
            Ok((
                TL::EMPTY,
                AstNode::Multiply {
                    left: Box::new(AstNode::Number(2.)),
                    right: Box::new(AstNode::NamedMatrix(MatrixName::new("M")))
                }
            ))
        );

        assert_eq!(
            parse_addition(TL::new(&[
                T::NamedMatrix(MatrixName::new("A")),
                T::Plus,
                T::NamedMatrix(MatrixName::new("B")),
            ])),
            Ok((
                TL::EMPTY,
                AstNode::Add {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
                    right: Box::new(AstNode::NamedMatrix(MatrixName::new("B")))
                }
            ))
        );
    }

    #[test]
    fn parse_compound_success() {
        // A + B * C
        assert_eq!(
            parse_expression(TL::new(&[
                T::NamedMatrix(MatrixName::new("A")),
                T::Plus,
                T::NamedMatrix(MatrixName::new("B")),
                T::Star,
                T::NamedMatrix(MatrixName::new("C")),
            ])),
            Ok((
                TL::EMPTY,
                AstNode::Add {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
                    right: Box::new(AstNode::Multiply {
                        left: Box::new(AstNode::NamedMatrix(MatrixName::new("B"))),
                        right: Box::new(AstNode::NamedMatrix(MatrixName::new("C")))
                    })
                }
            ))
        );

        // A * B + C
        assert_eq!(
            parse_expression(TL::new(&[
                T::NamedMatrix(MatrixName::new("A")),
                T::Star,
                T::NamedMatrix(MatrixName::new("B")),
                T::Plus,
                T::NamedMatrix(MatrixName::new("C")),
            ])),
            Ok((
                TL::EMPTY,
                AstNode::Add {
                    left: Box::new(AstNode::Multiply {
                        left: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
                        right: Box::new(AstNode::NamedMatrix(MatrixName::new("B")))
                    }),
                    right: Box::new(AstNode::NamedMatrix(MatrixName::new("C")))
                }
            ))
        );

        // A * (B + C)
        assert_eq!(
            parse_expression(TL::new(&[
                T::NamedMatrix(MatrixName::new("A")),
                T::Star,
                T::OpenParen,
                T::NamedMatrix(MatrixName::new("B")),
                T::Plus,
                T::NamedMatrix(MatrixName::new("C")),
                T::CloseParen,
            ])),
            Ok((
                TL::EMPTY,
                AstNode::Multiply {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
                    right: Box::new(AstNode::Add {
                        left: Box::new(AstNode::NamedMatrix(MatrixName::new("B"))),
                        right: Box::new(AstNode::NamedMatrix(MatrixName::new("C")))
                    })
                }
            ))
        );
    }
}
