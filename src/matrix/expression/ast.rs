//! This module handles abstract syntax trees for parsed matrix expressions.

use crate::{
    math::integer_power,
    matrix::{
        map::prelude::*, CannotAddDifferentDimensions, CannotMultiplyDifferentDimensions,
        Matrix2dOr3d, MatrixName,
    },
};
use approx::RelativeEq;
use glam::f64::{DMat2, DMat3};
use std::fmt;
use thiserror::Error;

/// A node in the tree. Also represents the tree itself, since the root is just a node.
#[derive(Clone, Debug, PartialEq)]
pub enum AstNode<'n> {
    /// Multiply two things together.
    Multiply {
        /// The value on the left of the multiplication.
        left: Box<Self>,
        /// The value on the right of the multiplication.
        right: Box<Self>,
    },

    /// Add two things together.
    Add {
        /// The value on the left of the addition.
        left: Box<Self>,
        /// The value on the right of the addition.
        right: Box<Self>,
    },

    /// Raise one thing to the power of another.
    Exponent {
        /// The base part of the exponentiation. The `b` in `b^p`.
        base: Box<Self>,

        /// The power part of the exponentiation. The `p` in `b^p`.
        ///
        /// The power must always evaluate to a number, and if the base is a matrix, then the
        /// power must be an integer.
        power: Box<Self>,
    },

    /// A real number.
    Number(f64),

    /// A named matrix. See [`MatrixName`].
    NamedMatrix(MatrixName<'n>),

    /// A rotation matrix, written in the expression like `rot(45)` or `rot(90)`.
    RotationMatrix {
        /// The number of degrees of rotation.
        degrees: f64,
    },

    /// An unnamed 2D matrix, written inline in the expression like `[1 2; 3 4]`.
    Anonymous2dMatrix(DMat2),

    /// An unnamed 3D matrix, written inline in the expression like `[1 2 3; 4 5 6; 7 8 9]`.
    Anonymous3dMatrix(DMat3),
}

/// Either a number or a [`Matrix2dOr3d`].
#[derive(Clone, Debug, PartialEq)]
pub enum NumberOrMatrix {
    /// A number.
    Number(f64),

    /// Either a [`DMat2`] or [`DMat3`].
    Matrix(Matrix2dOr3d),
}

impl NumberOrMatrix {
    /// Try to multiply.
    pub fn try_mul(self, rhs: Self) -> Result<Self, CannotMultiplyDifferentDimensions> {
        Ok(match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a * b),
            (Self::Number(a), Self::Matrix(b)) => Self::Matrix(a * b),
            (Self::Matrix(a), Self::Number(b)) => Self::Matrix(a * b),
            (Self::Matrix(a), Self::Matrix(b)) => Self::Matrix(Matrix2dOr3d::try_mul(a, b)?),
        })
    }

    /// Try to add.
    pub fn try_add(self, rhs: Self) -> Result<Self, EvaluationError> {
        Ok(match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a + b),
            (Self::Matrix(a), Self::Matrix(b)) => Self::Matrix(Matrix2dOr3d::try_add(a, b)?),
            _ => Err(CannotAddNumberAndMatrix)?,
        })
    }

    /// Try to raise one thing to the power of another.
    pub fn try_power(base: Self, power: Self) -> Result<Self, EvaluationError> {
        match (base, power) {
            (Self::Number(base), Self::Number(power)) => Ok(Self::Number(base.powf(power))),
            (Self::Matrix(Matrix2dOr3d::TwoD(base)), Self::Number(power)) => {
                if power.round().relative_eq(
                    &power,
                    0.000000001,
                    <f64 as RelativeEq>::default_max_relative(),
                ) {
                    let needs_invert = power < -0.0000000000001;
                    let power = power.round().abs() as u16;

                    let result = integer_power(base, power);

                    Ok(Self::Matrix(Matrix2dOr3d::TwoD(if needs_invert {
                        result.inverse()
                    } else {
                        result
                    })))
                } else {
                    Err(EvaluationError::CannotRaiseMatrixToNonInteger)
                }
            }
            (Self::Matrix(Matrix2dOr3d::ThreeD(base)), Self::Number(power)) => {
                if power.round().relative_eq(
                    &power,
                    0.000000001,
                    <f64 as RelativeEq>::default_max_relative(),
                ) {
                    let needs_invert = power < -0.0000000000001;
                    let power = power.round().abs() as u16;

                    let result = integer_power(base, power);

                    Ok(Self::Matrix(Matrix2dOr3d::ThreeD(if needs_invert {
                        result.inverse()
                    } else {
                        result
                    })))
                } else {
                    Err(EvaluationError::CannotRaiseMatrixToNonInteger)
                }
            }
            (_, Self::Matrix(_)) => panic!(concat!(
                "Anything raised to the power of a matrix is a fundamentally invalid AST. ",
                "The parser should error before this point."
            )),
        }
    }
}

/// Cannot add number and matrix.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub struct CannotAddNumberAndMatrix;

impl fmt::Display for CannotAddNumberAndMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot add number and matrix")
    }
}

/// An error which can be returned by [`AstNode::evaluate`].
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EvaluationError {
    /// Cannot multiply two different dimensions of matrices.
    #[error("{0}")]
    CannotMultiplyDifferentDimensions(#[from] CannotMultiplyDifferentDimensions),

    /// Cannot add two different dimensions of matrices.
    #[error("{0}")]
    CannotAddDifferentDimensions(#[from] CannotAddDifferentDimensions),

    /// Cannot add a number and a matrix.
    #[error("{0}")]
    CannotAddNumberAndMatrix(#[from] CannotAddNumberAndMatrix),

    /// Cannot raise a matrix to a non-integer number.
    #[error("Cannot raise a matrix to a non-integer number")]
    CannotRaiseMatrixToNonInteger,

    /// An error occurred when getting a value from the matrix map.
    #[error("{0}")]
    MatrixMapError(#[from] MatrixMapError),
}

impl<'n> AstNode<'n> {
    /// Evaluate this AST node by recursively evaulating whatever else needs to be evaluated.
    pub fn evaluate(self, map: &impl MatrixMap) -> Result<NumberOrMatrix, EvaluationError> {
        match self {
            Self::Multiply { left, right } => Ok(NumberOrMatrix::try_mul(
                left.evaluate(map)?,
                right.evaluate(map)?,
            )?),
            Self::Add { left, right } => Ok(NumberOrMatrix::try_add(
                left.evaluate(map)?,
                right.evaluate(map)?,
            )?),
            Self::Exponent { base, power } => Ok(NumberOrMatrix::try_power(
                base.evaluate(map)?,
                power.evaluate(map)?,
            )?),
            Self::Number(number) => Ok(NumberOrMatrix::Number(number)),
            Self::NamedMatrix(name) => Ok(NumberOrMatrix::Matrix(map.get(name)?.into())),
            Self::RotationMatrix { degrees } => Ok(NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(
                DMat2::from_angle(degrees.to_radians()),
            ))),
            Self::Anonymous2dMatrix(matrix) => {
                Ok(NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(matrix)))
            }
            Self::Anonymous3dMatrix(matrix) => {
                Ok(NumberOrMatrix::Matrix(Matrix2dOr3d::ThreeD(matrix)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::{assert_relative_eq, AbsDiffEq, RelativeEq};
    use glam::{DVec2, DVec3};
    use std::f64::consts::FRAC_1_SQRT_2;

    impl AbsDiffEq for NumberOrMatrix {
        type Epsilon = <f64 as AbsDiffEq>::Epsilon;

        fn default_epsilon() -> Self::Epsilon {
            <f64 as AbsDiffEq>::default_epsilon()
        }

        fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
            match (self, other) {
                (Self::Number(a), Self::Number(b)) => a.abs_diff_eq(b, epsilon),
                (Self::Matrix(a), Self::Matrix(b)) => a.abs_diff_eq(b, epsilon),
                _ => false,
            }
        }
    }

    impl RelativeEq for NumberOrMatrix {
        fn default_max_relative() -> Self::Epsilon {
            <f64 as RelativeEq>::default_max_relative()
        }

        fn relative_eq(
            &self,
            other: &Self,
            epsilon: Self::Epsilon,
            max_relative: Self::Epsilon,
        ) -> bool {
            match (self, other) {
                (Self::Number(a), Self::Number(b)) => a.relative_eq(b, epsilon, max_relative),
                (Self::Matrix(a), Self::Matrix(b)) => a.relative_eq(b, epsilon, max_relative),
                _ => false,
            }
        }
    }

    #[test]
    fn ast_node_evaluation_success() {
        let map2 = MatrixMap2::new();
        let map3 = MatrixMap3::new();

        // 10
        assert_relative_eq!(
            AstNode::evaluate(AstNode::Number(10.), &map2).unwrap(),
            NumberOrMatrix::Number(10.)
        );

        // 3.2 * 5
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Multiply {
                    left: Box::new(AstNode::Number(3.2)),
                    right: Box::new(AstNode::Number(5.))
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Number(16.)
        );

        // 1 + 2
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Add {
                    left: Box::new(AstNode::Number(1.)),
                    right: Box::new(AstNode::Number(2.))
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Number(3.)
        );

        // 3 * [2 -2.2; 1.5 10]
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Multiply {
                    left: Box::new(AstNode::Number(3.)),
                    right: Box::new(AstNode::Anonymous2dMatrix(DMat2::from_cols(
                        DVec2::new(2., 1.5),
                        DVec2::new(-2.2, 10.)
                    )))
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(DMat2::from_cols(
                DVec2::new(6., 4.5),
                DVec2::new(-6.6, 30.)
            )))
        );

        // [1 -2.34 2.3; 2.5 0 -0.5; 3.1 0.5 9.2] * ((1.2 + 2.3) * [2.3 -1.2 -3; 1.4 3 1; -3.2 -6.3 2.22])
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Multiply {
                    left: Box::new(AstNode::Anonymous3dMatrix(DMat3::from_cols(
                        DVec3::new(1., 2.5, 3.1),
                        DVec3::new(-2.34, 0., 0.5),
                        DVec3::new(2.3, -0.5, 9.2)
                    ))),
                    right: Box::new(AstNode::Multiply {
                        left: Box::new(AstNode::Add {
                            left: Box::new(AstNode::Number(1.2)),
                            right: Box::new(AstNode::Number(2.3))
                        }),
                        right: Box::new(AstNode::Anonymous3dMatrix(DMat3::from_cols(
                            DVec3::new(2.3, 1.4, -3.2),
                            DVec3::new(-1.2, 3., -6.3),
                            DVec3::new(-3., 1., 2.22)
                        )))
                    })
                },
                &map3
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::ThreeD(DMat3::from_cols(
                DVec3::new(-29.176, 25.725, -75.635),
                DVec3::new(-79.485, 0.525, -210.63),
                DVec3::new(-0.819, -30.135, 40.684)
            ))),
            epsilon = 0.00000000000001
        );

        // rot(45)
        assert_relative_eq!(
            AstNode::evaluate(AstNode::RotationMatrix { degrees: 45. }, &map2).unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(DMat2::from_cols(
                DVec2::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
                DVec2::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2)
            )))
        );

        // [1 2; 3 2] ^ (1 + 2)
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Anonymous2dMatrix(DMat2::from_cols(
                        DVec2::new(1., 3.),
                        DVec2::new(2., 2.)
                    ))),
                    power: Box::new(AstNode::Add {
                        left: Box::new(AstNode::Number(1.)),
                        right: Box::new(AstNode::Number(2.))
                    })
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(DMat2::from_cols(
                DVec2::new(25., 39.),
                DVec2::new(26., 38.)
            )))
        );

        // TODO: Test using named matrices from the map
    }

    #[test]
    fn ast_node_evaluation_failure() {
        let map2 = MatrixMap2::new();
        // let map3 = MatrixMap3::new();

        // 3 + [4 -1; -1.5 3]
        assert_eq!(
            AstNode::evaluate(
                AstNode::Add {
                    left: Box::new(AstNode::Number(3.)),
                    right: Box::new(AstNode::Anonymous2dMatrix(DMat2::from_cols(
                        DVec2::new(4., -1.5),
                        DVec2::new(-1., 3.)
                    )))
                },
                &map2
            ),
            Err(EvaluationError::CannotAddNumberAndMatrix(
                CannotAddNumberAndMatrix
            ))
        );

        // [1 4 7; 2 5 8; 3 6 9] * [4 -1; 1.5 3]
        assert_eq!(
            AstNode::evaluate(
                AstNode::Multiply {
                    left: Box::new(AstNode::Anonymous3dMatrix(DMat3::from_cols(
                        DVec3::new(1., 2., 3.),
                        DVec3::new(4., 5., 6.),
                        DVec3::new(7., 8., 9.)
                    ))),
                    right: Box::new(AstNode::Anonymous2dMatrix(DMat2::from_cols(
                        DVec2::new(4., -1.5),
                        DVec2::new(-1., 3.)
                    )))
                },
                &map2
            ),
            Err(EvaluationError::CannotMultiplyDifferentDimensions(
                CannotMultiplyDifferentDimensions
            ))
        );

        // [1 4 7; 2 5 8; 3 6 9] + [4 -1; 1.5 3]
        assert_eq!(
            AstNode::evaluate(
                AstNode::Add {
                    left: Box::new(AstNode::Anonymous3dMatrix(DMat3::from_cols(
                        DVec3::new(1., 2., 3.),
                        DVec3::new(4., 5., 6.),
                        DVec3::new(7., 8., 9.)
                    ))),
                    right: Box::new(AstNode::Anonymous2dMatrix(DMat2::from_cols(
                        DVec2::new(4., -1.5),
                        DVec2::new(-1., 3.)
                    )))
                },
                &map2
            ),
            Err(EvaluationError::CannotAddDifferentDimensions(
                CannotAddDifferentDimensions
            ))
        );

        // [1 0; 0 1] ^ 1.5
        assert_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Anonymous2dMatrix(DMat2::IDENTITY)),
                    power: Box::new(AstNode::Number(1.5))
                },
                &map2
            ),
            Err(EvaluationError::CannotRaiseMatrixToNonInteger)
        );
    }

    #[test]
    #[should_panic = "Anything raised to the power of a matrix is a fundamentally invalid AST"]
    fn ast_node_evaluation_raise_to_matrix() {
        AstNode::evaluate(
            AstNode::Exponent {
                base: Box::new(AstNode::Number(2.3)),
                power: Box::new(AstNode::Anonymous2dMatrix(DMat2::IDENTITY)),
            },
            &MatrixMap2::new(),
        )
        .ok();
    }
}
