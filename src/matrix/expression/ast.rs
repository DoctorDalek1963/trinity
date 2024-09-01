//! This module handles abstract syntax trees for parsed matrix expressions.

use crate::{
    math::integer_power,
    matrix::{map::prelude::*, Matrix2dOr3d, MatrixName},
};
use approx::RelativeEq;
use glam::f64::{DMat2, DMat3};
use thiserror::Error;

/// The epsilon value to use for relative comparisons.
const EPSILON: f64 = 0.000000001;

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

    /// Divide two things.
    Divide {
        /// The value on the left of the division.
        left: Box<Self>,
        /// The value on the right of the division.
        right: Box<Self>,
    },

    /// Add two things together.
    Add {
        /// The value on the left of the addition.
        left: Box<Self>,
        /// The value on the right of the addition.
        right: Box<Self>,
    },

    /// Negate another AST node.
    ///
    /// This node type is used to implement subtraction. The parser converts the string "A - B"
    /// into the AST (roughly) `Add { left: MatrixName("A"), right: Negate(MatrixName("B")) }`,
    /// equivalent to `A + (-B)`.
    Negate(Box<Self>),

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
    pub fn try_mul(self, rhs: Self) -> Result<Self, EvaluationError> {
        Ok(match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a * b),
            (Self::Number(a), Self::Matrix(b)) => Self::Matrix(a * b),
            (Self::Matrix(a), Self::Number(b)) => Self::Matrix(a * b),
            (Self::Matrix(a), Self::Matrix(b)) => Self::Matrix(
                Matrix2dOr3d::try_mul(a, b)
                    .ok_or(EvaluationError::CannotMultiplyDifferentDimensions)?,
            ),
        })
    }

    /// Try to divide.
    pub fn try_div(self, rhs: Self) -> Result<Self, EvaluationError> {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Ok(Self::Number(a / b)),
            (Self::Matrix(a), Self::Number(b)) => Ok(Self::Matrix(a * b.recip())),
            (_, Self::Matrix(_)) => Err(EvaluationError::CannotDivideByMatrix),
        }
    }

    /// Try to add.
    pub fn try_add(self, rhs: Self) -> Result<Self, EvaluationError> {
        Ok(match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a + b),
            (Self::Matrix(a), Self::Matrix(b)) => Self::Matrix(
                Matrix2dOr3d::try_add(a, b).ok_or(EvaluationError::CannotAddDifferentDimensions)?,
            ),
            _ => Err(EvaluationError::CannotAddNumberAndMatrix)?,
        })
    }

    /// Negate this number or matrix.
    pub fn negate(self) -> Self {
        match self {
            Self::Number(number) => Self::Number(-number),
            Self::Matrix(Matrix2dOr3d::TwoD(matrix)) => Self::Matrix(Matrix2dOr3d::TwoD(-matrix)),
            Self::Matrix(Matrix2dOr3d::ThreeD(matrix)) => {
                Self::Matrix(Matrix2dOr3d::ThreeD(-matrix))
            }
        }
    }

    /// Try to raise one thing to the power of another.
    pub fn try_power(base: Self, power: Self) -> Result<Self, EvaluationError> {
        match (base, power) {
            (Self::Number(base), Self::Number(power)) => Ok(Self::Number(base.powf(power))),
            (Self::Matrix(Matrix2dOr3d::TwoD(base)), Self::Number(power)) => {
                if power.round().relative_eq(
                    &power,
                    EPSILON,
                    <f64 as RelativeEq>::default_max_relative(),
                ) {
                    let needs_invert = power.round() < 0.;
                    let power = power.round().abs() as u16;

                    let result = integer_power(base, power);

                    Ok(Self::Matrix(Matrix2dOr3d::TwoD(if needs_invert {
                        if !result.determinant().relative_eq(
                            &0.,
                            EPSILON,
                            <f64 as RelativeEq>::default_max_relative(),
                        ) {
                            result.inverse()
                        } else {
                            Err(EvaluationError::CannotInvertSingularMatrix)?
                        }
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
                    let needs_invert = power.round() < 0.;
                    let power = power.round().abs() as u16;

                    let result = integer_power(base, power);

                    Ok(Self::Matrix(Matrix2dOr3d::ThreeD(if needs_invert {
                        if !result.determinant().relative_eq(
                            &0.,
                            EPSILON,
                            <f64 as RelativeEq>::default_max_relative(),
                        ) {
                            result.inverse()
                        } else {
                            Err(EvaluationError::CannotInvertSingularMatrix)?
                        }
                    } else {
                        result
                    })))
                } else {
                    Err(EvaluationError::CannotRaiseMatrixToNonInteger)
                }
            }
            (_, Self::Matrix(_)) => Err(EvaluationError::CannotRaiseToMatrix),
        }
    }

    /// Try to transpose this thing.
    pub fn try_transpose(self) -> Result<Self, EvaluationError> {
        match self {
            Self::Number(_) => Err(EvaluationError::CannotTransposeNumber),
            Self::Matrix(Matrix2dOr3d::TwoD(matrix)) => {
                Ok(Self::Matrix(Matrix2dOr3d::TwoD(matrix.transpose())))
            }
            Self::Matrix(Matrix2dOr3d::ThreeD(matrix)) => {
                Ok(Self::Matrix(Matrix2dOr3d::ThreeD(matrix.transpose())))
            }
        }
    }
}

/// An error which can be returned by [`AstNode::evaluate`].
#[allow(
    missing_docs,
    reason = "All variants impl Display and most are obvious from the name"
)]
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EvaluationError {
    #[error("Cannot multiply two matrices of different dimensions")]
    CannotMultiplyDifferentDimensions,

    #[error("Cannot add two matrices of different dimensions")]
    CannotAddDifferentDimensions,

    #[error("Cannot add a number and a matrix")]
    CannotAddNumberAndMatrix,

    #[error("Cannot raise a matrix to a non-integer number")]
    CannotRaiseMatrixToNonInteger,

    #[error("Cannot raise anything to the power of a matrix")]
    CannotRaiseToMatrix,

    #[error("Cannot divide by a matrix")]
    CannotDivideByMatrix,

    #[error("Cannot invert a singular (determinant 0) matrix")]
    CannotInvertSingularMatrix,

    #[error("Cannot transpose a scalar number")]
    CannotTransposeNumber,

    /// An error occurred when getting a value from the matrix map.
    #[error("{0}")]
    MatrixMapError(#[from] MatrixMapError),
}

impl<'n> AstNode<'n> {
    /// Evaluate this AST node by recursively evaulating whatever else needs to be evaluated.
    pub fn evaluate(self, map: &impl MatrixMap) -> Result<NumberOrMatrix, EvaluationError> {
        match self {
            Self::Multiply { left, right } => {
                NumberOrMatrix::try_mul(left.evaluate(map)?, right.evaluate(map)?)
            }
            Self::Divide { left, right } => {
                NumberOrMatrix::try_div(left.evaluate(map)?, right.evaluate(map)?)
            }
            Self::Add { left, right } => {
                NumberOrMatrix::try_add(left.evaluate(map)?, right.evaluate(map)?)
            }
            Self::Negate(term) => Ok(NumberOrMatrix::negate(term.evaluate(map)?)),
            Self::Exponent { base, power } => {
                if *power == Self::NamedMatrix(MatrixName::new("T")) {
                    NumberOrMatrix::try_transpose(base.evaluate(map)?)
                } else {
                    NumberOrMatrix::try_power(base.evaluate(map)?, power.evaluate(map)?)
                }
            }
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

    /// Convert this AST node into an expression string.
    pub fn to_expression_string(&self) -> String {
        self.internal_to_expression_string(true)
    }

    /// The internal implementation of [`AstNode::to_expression_string`]. The `top_level` parameter
    /// is used to control parentheses.
    fn internal_to_expression_string(&self, top_level: bool) -> String {
        match self {
            Self::Multiply { left, right } => {
                let left = left.internal_to_expression_string(false);
                let right = right.internal_to_expression_string(false);
                let string = format!("{left} * {right}");
                if !top_level {
                    format!("({string})")
                } else {
                    string
                }
            }
            Self::Divide { left, right } => {
                let left = left.internal_to_expression_string(false);
                let right = right.internal_to_expression_string(false);
                let string = format!("{left} / {right}");
                if !top_level {
                    format!("({string})")
                } else {
                    string
                }
            }
            Self::Add { left, right } => {
                let left = left.internal_to_expression_string(false);
                let right = right.internal_to_expression_string(false);
                let string = format!("{left} + {right}");
                if !top_level {
                    format!("({string})")
                } else {
                    string
                }
            }
            Self::Negate(term) => {
                let term = term.internal_to_expression_string(false);
                if !top_level {
                    format!("(-{term})")
                } else {
                    format!("-{term}")
                }
            }
            Self::Exponent { base, power } => {
                let base = base.internal_to_expression_string(false);
                // The braces also act as parens, so we can treat the power as if it were top-level
                let power = power.internal_to_expression_string(true);
                let string = format!("{base} ^ {{{power}}}");
                if !top_level {
                    format!("({string})")
                } else {
                    string
                }
            }
            Self::Number(number) => number.to_string(),
            Self::NamedMatrix(MatrixName { name }) => name.to_string(),
            Self::RotationMatrix { degrees } => format!("rot({degrees})"),
            Self::Anonymous2dMatrix(DMat2 { x_axis, y_axis }) => {
                format!("[{} {}; {} {}]", x_axis.x, y_axis.x, x_axis.y, y_axis.y)
            }

            // This is utterly bizarre, but cargo tarpaulin complains about this if it's formatted
            // nicely (ie. across multiple lines). Either we have this one ugly line here, or code
            // coverage takes a needless hit.
            #[rustfmt::skip]
            Self::Anonymous3dMatrix(DMat3 { x_axis, y_axis, z_axis }) => format!("[{} {} {}; {} {} {}; {} {} {}]", x_axis.x, y_axis.x, z_axis.x, x_axis.y, y_axis.y, z_axis.y, x_axis.z, y_axis.z, z_axis.z),
        }
    }

    /// Get all the named matrices that are referenced in this AST.
    pub fn named_matrices(&self) -> Vec<MatrixName> {
        match self {
            Self::Multiply { left, right } => left
                .named_matrices()
                .into_iter()
                .chain(right.named_matrices())
                .collect(),
            Self::Divide { left, right } => left
                .named_matrices()
                .into_iter()
                .chain(right.named_matrices())
                .collect(),
            Self::Add { left, right } => left
                .named_matrices()
                .into_iter()
                .chain(right.named_matrices())
                .collect(),
            Self::Negate(term) => term.named_matrices(),
            Self::Exponent { base, power } => {
                if **power == AstNode::NamedMatrix(MatrixName::new("T")) {
                    base.named_matrices()
                } else {
                    base.named_matrices()
                        .into_iter()
                        .chain(power.named_matrices())
                        .collect()
                }
            }
            Self::Number(_) => vec![],
            Self::NamedMatrix(matrix) => vec![*matrix],
            Self::RotationMatrix { degrees: _ } => vec![],
            Self::Anonymous2dMatrix(_) => vec![],
            Self::Anonymous3dMatrix(_) => vec![],
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
        let mut map2 = MatrixMap2::new();
        let mut map3 = MatrixMap3::new();

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

        // [1 2; 3 4] ^ -1
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Anonymous2dMatrix(DMat2::from_cols(
                        DVec2::new(1., 3.),
                        DVec2::new(2., 4.)
                    ))),
                    power: Box::new(AstNode::Negate(Box::new(AstNode::Number(1.))))
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(DMat2::from_cols(
                DVec2::new(-2., 1.5),
                DVec2::new(1., -0.5)
            )))
        );

        // [1 2 3; 4 5 6; 1 2 4] ^ -1
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Anonymous3dMatrix(DMat3::from_cols(
                        DVec3::new(1., 4., 1.),
                        DVec3::new(2., 5., 2.),
                        DVec3::new(3., 6., 4.),
                    ))),
                    power: Box::new(AstNode::Negate(Box::new(AstNode::Number(1.))))
                },
                &map3
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::ThreeD(DMat3::from_cols(
                DVec3::new(-(2. + 2. / 3.), 3. + 1. / 3., -1.),
                DVec3::new(2. / 3., -1. / 3., 0.),
                DVec3::new(1., -2., 1.),
            )))
        );

        map2.set(
            MatrixName::new("M"),
            DMat2::from_cols(DVec2::new(1., 3.), DVec2::new(2., 4.)),
        )
        .expect("Should be able to set 2D matrix M");

        map3.set(
            MatrixName::new("X"),
            DMat3::from_cols(
                DVec3::new(1., 4., 1.),
                DVec3::new(2., 5., 2.),
                DVec3::new(3., 6., 4.),
            ),
        )
        .expect("Should be able to set 3D matrix X");

        // M * 3
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Multiply {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                    right: Box::new(AstNode::Number(3.))
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(DMat2::from_cols(
                DVec2::new(3., 9.),
                DVec2::new(6., 12.)
            )))
        );

        // X ^ (2 ^ {1 + 1})
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::NamedMatrix(MatrixName::new("X"))),
                    power: Box::new(AstNode::Exponent {
                        base: Box::new(AstNode::Number(2.)),
                        power: Box::new(AstNode::Add {
                            left: Box::new(AstNode::Number(1.)),
                            right: Box::new(AstNode::Number(1.))
                        })
                    })
                },
                &map3
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::ThreeD(DMat3::from_cols(
                DVec3::new(1035., 2568., 1159.),
                DVec3::new(1566., 3885., 1754.),
                DVec3::new(2349., 5826., 2632.),
            )))
        );

        // X * (X + X)
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Multiply {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("X"))),
                    right: Box::new(AstNode::Add {
                        left: Box::new(AstNode::NamedMatrix(MatrixName::new("X"))),
                        right: Box::new(AstNode::NamedMatrix(MatrixName::new("X")))
                    })
                },
                &map3
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::ThreeD(DMat3::from_cols(
                DVec3::new(24., 60., 26.),
                DVec3::new(36., 90., 40.),
                DVec3::new(54., 132., 62.),
            )))
        );

        // X * (1 + 2)
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Multiply {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("X"))),
                    right: Box::new(AstNode::Add {
                        left: Box::new(AstNode::Number(1.)),
                        right: Box::new(AstNode::Number(2.))
                    })
                },
                &map3
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::ThreeD(DMat3::from_cols(
                DVec3::new(3., 12., 3.),
                DVec3::new(6., 15., 6.),
                DVec3::new(9., 18., 12.),
            )))
        );

        // M * [1 0; 0 1] + [0 2; 3 0]
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Add {
                    left: Box::new(AstNode::Multiply {
                        left: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                        right: Box::new(AstNode::Anonymous2dMatrix(DMat2::IDENTITY))
                    }),
                    right: Box::new(AstNode::Anonymous2dMatrix(DMat2::from_cols(
                        DVec2::new(0., 3.),
                        DVec2::new(2., 0.)
                    )))
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(DMat2::from_cols(
                DVec2::new(1., 6.),
                DVec2::new(4., 4.),
            )))
        );

        // 3 / 4
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Divide {
                    left: Box::new(AstNode::Number(3.)),
                    right: Box::new(AstNode::Number(4.))
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Number(0.75)
        );

        // [1 2; 3 4] / 4
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Divide {
                    left: Box::new(AstNode::Anonymous2dMatrix(DMat2::from_cols(
                        DVec2::new(1., 3.),
                        DVec2::new(2., 4.)
                    ))),
                    right: Box::new(AstNode::Number(4.))
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(DMat2::from_cols(
                DVec2::new(0.25, 0.75),
                DVec2::new(0.5, 1.)
            )))
        );

        // 2 + (-1)
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Add {
                    left: Box::new(AstNode::Number(2.)),
                    right: Box::new(AstNode::Negate(Box::new(AstNode::Number(1.))))
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Number(1.)
        );

        // M + (2 * (-M))
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Add {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                    right: Box::new(AstNode::Multiply {
                        left: Box::new(AstNode::Number(2.)),
                        right: Box::new(AstNode::Negate(Box::new(AstNode::NamedMatrix(
                            MatrixName::new("M")
                        ))))
                    })
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(DMat2::from_cols(
                DVec2::new(-1., -3.),
                DVec2::new(-2., -4.)
            )))
        );

        // X + (3.5 * (-X))
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Add {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("X"))),
                    right: Box::new(AstNode::Multiply {
                        left: Box::new(AstNode::Number(3.5)),
                        right: Box::new(AstNode::Negate(Box::new(AstNode::NamedMatrix(
                            MatrixName::new("X")
                        ))))
                    })
                },
                &map3
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::ThreeD(DMat3::from_cols(
                DVec3::new(-2.5, -10., -2.5),
                DVec3::new(-5., -12.5, -5.),
                DVec3::new(-7.5, -15., -10.),
            )))
        );

        // [1 2; 3 4] ^ T
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Anonymous2dMatrix(DMat2::from_cols(
                        DVec2::new(1., 3.),
                        DVec2::new(2., 4.)
                    ))),
                    power: Box::new(AstNode::NamedMatrix(MatrixName::new("T")))
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(DMat2::from_cols(
                DVec2::new(1., 2.),
                DVec2::new(3., 4.)
            )))
        );

        // [1 2 3; 4 5 6; 7 8 9] ^ T
        assert_relative_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Anonymous3dMatrix(DMat3::from_cols(
                        DVec3::new(1., 4., 7.),
                        DVec3::new(2., 5., 8.),
                        DVec3::new(3., 6., 9.),
                    ))),
                    power: Box::new(AstNode::NamedMatrix(MatrixName::new("T")))
                },
                &map2
            )
            .unwrap(),
            NumberOrMatrix::Matrix(Matrix2dOr3d::ThreeD(DMat3::from_cols(
                DVec3::new(1., 2., 3.),
                DVec3::new(4., 5., 6.),
                DVec3::new(7., 8., 9.),
            )))
        );
    }

    #[test]
    fn ast_node_evaluation_failure() {
        let map2 = MatrixMap2::new();
        let map3 = MatrixMap3::new();

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
            Err(EvaluationError::CannotAddNumberAndMatrix)
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
            Err(EvaluationError::CannotMultiplyDifferentDimensions)
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
            Err(EvaluationError::CannotAddDifferentDimensions)
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

        // [1 0 0; 0 1 0; 0 0 1] ^ -3.7
        assert_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Anonymous3dMatrix(DMat3::IDENTITY)),
                    power: Box::new(AstNode::Negate(Box::new(AstNode::Number(3.7))))
                },
                &map3
            ),
            Err(EvaluationError::CannotRaiseMatrixToNonInteger)
        );

        // 2 / [1 2; 3 4]
        assert_eq!(
            AstNode::evaluate(
                AstNode::Divide {
                    left: Box::new(AstNode::Number(2.)),
                    right: Box::new(AstNode::Anonymous2dMatrix(DMat2::from_cols(
                        DVec2::new(1., 3.),
                        DVec2::new(2., 4.)
                    )))
                },
                &map2
            ),
            Err(EvaluationError::CannotDivideByMatrix)
        );

        // [1 0; 0 1] / [1 2; 3 4]
        assert_eq!(
            AstNode::evaluate(
                AstNode::Divide {
                    left: Box::new(AstNode::Anonymous2dMatrix(DMat2::IDENTITY)),
                    right: Box::new(AstNode::Anonymous2dMatrix(DMat2::from_cols(
                        DVec2::new(1., 3.),
                        DVec2::new(2., 4.)
                    )))
                },
                &map2
            ),
            Err(EvaluationError::CannotDivideByMatrix)
        );

        assert_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Anonymous2dMatrix(DMat2::ZERO)),
                    power: Box::new(AstNode::Negate(Box::new(AstNode::Number(1.))))
                },
                &map2
            ),
            Err(EvaluationError::CannotInvertSingularMatrix)
        );

        assert_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Anonymous3dMatrix(DMat3::ZERO)),
                    power: Box::new(AstNode::Negate(Box::new(AstNode::Number(1.))))
                },
                &map3
            ),
            Err(EvaluationError::CannotInvertSingularMatrix)
        );

        assert_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Number(2.3)),
                    power: Box::new(AstNode::Anonymous2dMatrix(DMat2::IDENTITY)),
                },
                &map2,
            ),
            Err(EvaluationError::CannotRaiseToMatrix)
        );

        assert_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Anonymous3dMatrix(DMat3::IDENTITY)),
                    power: Box::new(AstNode::Anonymous3dMatrix(DMat3::IDENTITY)),
                },
                &map3,
            ),
            Err(EvaluationError::CannotRaiseToMatrix)
        );

        // (1 + 2) ^ T
        assert_eq!(
            AstNode::evaluate(
                AstNode::Exponent {
                    base: Box::new(AstNode::Add {
                        left: Box::new(AstNode::Number(1.)),
                        right: Box::new(AstNode::Number(2.))
                    }),
                    power: Box::new(AstNode::NamedMatrix(MatrixName::new("T")))
                },
                &map2
            ),
            Err(EvaluationError::CannotTransposeNumber)
        );
    }

    #[test]
    fn ast_node_to_expression_string() {
        assert_eq!(
            AstNode::to_expression_string(&AstNode::Multiply {
                left: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                right: Box::new(AstNode::Add {
                    left: Box::new(AstNode::Number(1.)),
                    right: Box::new(AstNode::Number(2.))
                })
            }),
            "M * (1 + 2)"
        );

        assert_eq!(
            AstNode::to_expression_string(&AstNode::Exponent {
                base: Box::new(AstNode::NamedMatrix(MatrixName::new("M"))),
                power: Box::new(AstNode::Number(2.))
            }),
            "M ^ {2}"
        );

        assert_eq!(
            AstNode::to_expression_string(&AstNode::Exponent {
                base: Box::new(AstNode::RotationMatrix { degrees: 45. }),
                power: Box::new(AstNode::Add {
                    left: Box::new(AstNode::Multiply {
                        left: Box::new(AstNode::Number(0.)),
                        right: Box::new(AstNode::NamedMatrix(MatrixName::new("X")))
                    }),
                    right: Box::new(AstNode::Number(1.))
                })
            }),
            "rot(45) ^ {(0 * X) + 1}"
        );

        assert_eq!(
            AstNode::to_expression_string(&AstNode::Multiply {
                left: Box::new(AstNode::Exponent {
                    base: Box::new(AstNode::Anonymous2dMatrix(DMat2::IDENTITY)),
                    power: Box::new(AstNode::Negate(Box::new(AstNode::Number(1.))))
                }),
                right: Box::new(AstNode::Anonymous3dMatrix(DMat3::IDENTITY))
            }),
            "([1 0; 0 1] ^ {-1}) * [1 0 0; 0 1 0; 0 0 1]"
        );

        assert_eq!(
            AstNode::to_expression_string(&AstNode::Anonymous3dMatrix(DMat3::from_cols(
                DVec3::new(1., 5., -3.),
                DVec3::new(2., 3., 4.),
                DVec3::new(3., 1., 2.),
            ))),
            "[1 2 3; 5 3 1; -3 4 2]"
        );

        assert_eq!(
            AstNode::to_expression_string(&AstNode::Add {
                left: Box::new(AstNode::Multiply {
                    left: Box::new(AstNode::Divide {
                        left: Box::new(AstNode::Number(2.)),
                        right: Box::new(AstNode::Number(3.))
                    }),
                    right: Box::new(AstNode::NamedMatrix(MatrixName::new("M")))
                }),
                right: Box::new(AstNode::Divide {
                    left: Box::new(AstNode::NamedMatrix(MatrixName::new("X"))),
                    right: Box::new(AstNode::Number(4.))
                })
            }),
            "((2 / 3) * M) + (X / 4)"
        );

        assert_eq!(
            AstNode::to_expression_string(&AstNode::Divide {
                left: Box::new(AstNode::Number(1.)),
                right: Box::new(AstNode::Add {
                    left: Box::new(AstNode::Number(1.)),
                    right: Box::new(AstNode::Number(1.))
                })
            }),
            "1 / (1 + 1)"
        );

        assert_eq!(
            AstNode::to_expression_string(&AstNode::Negate(Box::new(AstNode::NamedMatrix(
                MatrixName::new("M")
            )))),
            "-M"
        );

        assert_eq!(
            AstNode::to_expression_string(&AstNode::Add {
                left: Box::new(AstNode::Number(2.)),
                right: Box::new(AstNode::Negate(Box::new(AstNode::Number(3.))))
            }),
            "2 + (-3)"
        );
    }

    #[test]
    fn ast_node_named_matrices() {
        assert_eq!(AstNode::named_matrices(&AstNode::Number(1.)), vec![]);

        // A + ([1 0; 0 1] ^ T) * (-[1 0 0; 0 1 0; 0 0 1])
        // The only named matrix should be A since the T is part of a transposition
        assert_eq!(
            AstNode::named_matrices(&AstNode::Add {
                left: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
                right: Box::new(AstNode::Multiply {
                    left: Box::new(AstNode::Exponent {
                        base: Box::new(AstNode::Anonymous2dMatrix(DMat2::IDENTITY)),
                        power: Box::new(AstNode::NamedMatrix(MatrixName::new("T")))
                    }),
                    right: Box::new(AstNode::Negate(Box::new(AstNode::Anonymous3dMatrix(
                        DMat3::IDENTITY
                    ))))
                })
            }),
            vec![MatrixName::new("A")]
        );

        // T + A
        assert_eq!(
            AstNode::named_matrices(&AstNode::Add {
                left: Box::new(AstNode::NamedMatrix(MatrixName::new("T"))),
                right: Box::new(AstNode::NamedMatrix(MatrixName::new("A"))),
            }),
            vec![MatrixName::new("T"), MatrixName::new("A")]
        );
    }
}
