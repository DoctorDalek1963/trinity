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

#[cfg(test)]
mod tests {
    use super::ast::AstNode;
    use super::*;
    use crate::matrix::MatrixName;

    #[test]
    fn parse_expression_from_string_success() {
        assert_eq!(
            parse_expression_from_string("A + B / C"),
            Ok(AstNode::Add {
                left: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
                right: Box::new(AstNode::Divide {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("B"))),
                    right: Box::new(AstNode::NamedMatrix(MatrixName::new("C")))
                })
            })
        );

        assert_eq!(
            parse_expression_from_string("2 - 1"),
            Ok(AstNode::Add {
                left: Box::new(AstNode::Number(2.)),
                right: Box::new(AstNode::Negate(Box::new(AstNode::Number(1.))))
            })
        );

        assert_eq!(
            parse_expression_from_string("2 * -1"),
            Ok(AstNode::Multiply {
                left: Box::new(AstNode::Number(2.)),
                right: Box::new(AstNode::Negate(Box::new(AstNode::Number(1.))))
            })
        );

        assert_eq!(
            parse_expression_from_string("-1"),
            Ok(AstNode::Negate(Box::new(AstNode::Number(1.))))
        );

        assert_eq!(
            parse_expression_from_string("A + B ^ T * M ^ {-1} / 2"),
            Ok(AstNode::Add {
                left: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
                right: Box::new(AstNode::Multiply {
                    left: Box::new(AstNode::Exponent {
                        base: Box::new(AstNode::NamedMatrix(MatrixName::new("B"))),
                        power: Box::new(AstNode::NamedMatrix(MatrixName::new("T")))
                    }),
                    right: Box::new(AstNode::Divide {
                        left: Box::new(AstNode::Exponent {
                            base: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                            power: Box::new(AstNode::Negate(Box::new(AstNode::Number(1.))))
                        }),
                        right: Box::new(AstNode::Number(2.))
                    })
                })
            })
        );

        assert_eq!(
            parse_expression_from_string("(2*M + 3*X^-1) * (D/3/2)"),
            Ok(AstNode::Multiply {
                left: Box::new(AstNode::Add {
                    left: Box::new(AstNode::Multiply {
                        left: Box::new(AstNode::Number(2.)),
                        right: Box::new(AstNode::NamedMatrix(MatrixName::new("M")))
                    }),
                    right: Box::new(AstNode::Multiply {
                        left: Box::new(AstNode::Number(3.)),
                        right: Box::new(AstNode::Exponent {
                            base: Box::new(AstNode::NamedMatrix(MatrixName::new("X"))),
                            power: Box::new(AstNode::Negate(Box::new(AstNode::Number(1.))))
                        })
                    })
                }),
                right: Box::new(AstNode::Divide {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("D"))),
                    right: Box::new(AstNode::Divide {
                        left: Box::new(AstNode::Number(3.)),
                        right: Box::new(AstNode::Number(2.))
                    })
                })
            })
        );

        assert_eq!(
            parse_expression_from_string("M / 2 + B ^ 2 * rot(90)"),
            Ok(AstNode::Add {
                left: Box::new(AstNode::Divide {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                    right: Box::new(AstNode::Number(2.))
                }),
                right: Box::new(AstNode::Multiply {
                    left: Box::new(AstNode::Exponent {
                        base: Box::new(AstNode::NamedMatrix(MatrixName::new("B"))),
                        power: Box::new(AstNode::Number(2.))
                    }),
                    right: Box::new(AstNode::RotationMatrix { degrees: 90. })
                })
            })
        );

        assert_eq!(
            parse_expression_from_string("AB"),
            Ok(AstNode::Multiply {
                left: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
                right: Box::new(AstNode::NamedMatrix(MatrixName::new("B")))
            })
        );

        assert_eq!(
            parse_expression_from_string("2ABc"),
            Ok(AstNode::Multiply {
                left: Box::new(AstNode::Number(2.)),
                right: Box::new(AstNode::Multiply {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
                    right: Box::new(AstNode::NamedMatrix(MatrixName::new("Bc")))
                })
            })
        );

        assert_eq!(
            parse_expression_from_string("3M - 2X"),
            Ok(AstNode::Add {
                left: Box::new(AstNode::Multiply {
                    left: Box::new(AstNode::Number(3.)),
                    right: Box::new(AstNode::NamedMatrix(MatrixName::new("M")))
                }),
                right: Box::new(AstNode::Negate(Box::new(AstNode::Multiply {
                    left: Box::new(AstNode::Number(2.)),
                    right: Box::new(AstNode::NamedMatrix(MatrixName::new("X")))
                })))
            })
        );

        assert_eq!(
            parse_expression_from_string("3M(-2X)"),
            Ok(AstNode::Multiply {
                left: Box::new(AstNode::Number(3.)),
                right: Box::new(AstNode::Multiply {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                    right: Box::new(AstNode::Multiply {
                        left: Box::new(AstNode::Negate(Box::new(AstNode::Number(2.)))),
                        right: Box::new(AstNode::NamedMatrix(MatrixName::new("X")))
                    })
                })
            })
        );
    }

    #[test]
    fn parse_expression_from_string_abc() {
        assert_eq!(
            parse_expression_from_string("ABC"),
            Ok(AstNode::Multiply {
                left: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
                right: Box::new(AstNode::Multiply {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("B"))),
                    right: Box::new(AstNode::NamedMatrix(MatrixName::new("C")))
                })
            })
        );

        assert_eq!(
            parse_expression_from_string("ABc"),
            Ok(AstNode::Multiply {
                left: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
                right: Box::new(AstNode::NamedMatrix(MatrixName::new("Bc")))
            })
        );

        assert_eq!(
            parse_expression_from_string("AbC"),
            Ok(AstNode::Multiply {
                left: Box::new(AstNode::NamedMatrix(MatrixName::new("Ab"))),
                right: Box::new(AstNode::NamedMatrix(MatrixName::new("C")))
            })
        );

        assert_eq!(
            parse_expression_from_string("Abc"),
            Ok(AstNode::NamedMatrix(MatrixName::new("Abc")))
        );

        assert_eq!(
            parse_expression_from_string("aBC"),
            Err(TokeniseOrParseError::TokeniseError(
                tokenise::TokeniseError::NomError {
                    nom_error: nom::Err::Error(nom::error::Error::new(
                        "aBC",
                        nom::error::ErrorKind::MultiSpace
                    ))
                }
            ))
        );

        assert_eq!(
            parse_expression_from_string("aBc"),
            Err(TokeniseOrParseError::TokeniseError(
                tokenise::TokeniseError::NomError {
                    nom_error: nom::Err::Error(nom::error::Error::new(
                        "aBc",
                        nom::error::ErrorKind::MultiSpace
                    ))
                }
            ))
        );

        assert_eq!(
            parse_expression_from_string("abC"),
            Err(TokeniseOrParseError::TokeniseError(
                tokenise::TokeniseError::NomError {
                    nom_error: nom::Err::Error(nom::error::Error::new(
                        "abC",
                        nom::error::ErrorKind::MultiSpace
                    ))
                }
            ))
        );

        assert_eq!(
            parse_expression_from_string("abc"),
            Err(TokeniseOrParseError::TokeniseError(
                tokenise::TokeniseError::NomError {
                    nom_error: nom::Err::Error(nom::error::Error::new(
                        "abc",
                        nom::error::ErrorKind::MultiSpace
                    ))
                }
            ))
        );
    }

    #[test]
    fn parse_expression_from_string_failure() {
        use super::{
            parser::ParseError,
            tokenise::{Token, TokeniseError},
        };

        assert_eq!(
            parse_expression_from_string(""),
            Err(TokeniseOrParseError::TokeniseError(
                TokeniseError::NomError {
                    nom_error: nom::Err::Error(nom::error::Error::new(
                        "",
                        nom::error::ErrorKind::MultiSpace
                    ))
                }
            ))
        );

        assert_eq!(
            parse_expression_from_string("2 @ M"),
            Err(TokeniseOrParseError::TokeniseError(
                TokeniseError::UnconsumedInput("@ M")
            ))
        );

        assert_eq!(
            parse_expression_from_string("C++"),
            Err(TokeniseOrParseError::ParseError(ParseError::NomError(
                nom::Err::Error(nom::error::Error::new(
                    vec![Token::Plus],
                    nom::error::ErrorKind::Tag
                ))
            )))
        );

        assert_eq!(
            parse_expression_from_string("[1 2 3 4]"),
            Err(TokeniseOrParseError::ParseError(ParseError::NomError(
                nom::Err::Error(nom::error::Error::new(
                    vec![
                        Token::OpenSquareBracket,
                        Token::Number(1.0),
                        Token::Number(2.0),
                        Token::Number(3.0),
                        Token::Number(4.0),
                        Token::CloseSquareBracket
                    ],
                    nom::error::ErrorKind::Tag
                ))
            )))
        );

        assert_eq!(
            parse_expression_from_string("[1"),
            Err(TokeniseOrParseError::ParseError(ParseError::NomError(
                nom::Err::Error(nom::error::Error::new(
                    vec![Token::OpenSquareBracket, Token::Number(1.0),],
                    nom::error::ErrorKind::Tag
                ))
            )))
        );
    }
}
