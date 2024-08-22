//! This module handles abstract syntax trees for parsed matrix expressions.

use std::fmt;

use crate::matrix::{
    CannotAddDifferentDimensions, CannotMultiplyDifferentDimensions, Matrix2dOr3d, MatrixMap,
    MatrixName,
};
use glam::f64::{DMat2, DMat3};
use thiserror::Error;

/// A node in the tree. Also represents the tree itself, since the root is just a node.
#[derive(Clone, Debug, PartialEq)]
pub enum AstNode<'n> {
    Multiply { left: Box<Self>, right: Box<Self> },
    Add { left: Box<Self>, right: Box<Self> },
    Exponent { base: Box<Self>, power: Box<Self> },
    Number { number: f64 },
    NamedMatrix { name: MatrixName<'n> },
    RotationMatrix { degrees: f64 },
    Anonymous2dMatrix { matrix: DMat2 },
    Anonymous3dMatrix { matrix: DMat3 },
}

/// Either a number or a [`Matrix2dOr3d`].
#[derive(Clone, Debug, PartialEq)]
pub enum NumberOrMatrix {
    Number(f64),
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
    #[error("{0}")]
    CannotMultiplyDifferentDimensions(#[from] CannotMultiplyDifferentDimensions),

    #[error("{0}")]
    CannotAddDifferentDimensions(#[from] CannotAddDifferentDimensions),

    #[error("{0}")]
    CannotAddNumberAndMatrix(#[from] CannotAddNumberAndMatrix),
}

impl<'n> AstNode<'n> {
    /// Evaluate this AST node by recursively evaulating whatever else needs to be evaluated.
    pub fn evaluate(self, map: &MatrixMap) -> Result<NumberOrMatrix, EvaluationError> {
        match self {
            Self::Multiply { left, right } => Ok(NumberOrMatrix::try_mul(
                left.evaluate(map)?,
                right.evaluate(map)?,
            )?),
            Self::Add { left, right } => Ok(NumberOrMatrix::try_add(
                left.evaluate(map)?,
                right.evaluate(map)?,
            )?),
            Self::Exponent { base, power } => todo!(),
            Self::Number { number } => Ok(NumberOrMatrix::Number(number)),
            Self::NamedMatrix { name } => todo!(),
            Self::RotationMatrix { degrees } => todo!(),
            Self::Anonymous2dMatrix { matrix } => {
                Ok(NumberOrMatrix::Matrix(Matrix2dOr3d::TwoD(matrix)))
            }
            Self::Anonymous3dMatrix { matrix } => {
                Ok(NumberOrMatrix::Matrix(Matrix2dOr3d::ThreeD(matrix)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_node_evaluation() {
        let map = MatrixMap::new();

        assert_eq!(
            AstNode::evaluate(AstNode::Number { number: 10. }, &map),
            Ok(NumberOrMatrix::Number(10.))
        );

        assert_eq!(
            AstNode::evaluate(
                AstNode::Multiply {
                    left: Box::new(AstNode::Number { number: 3.2 }),
                    right: Box::new(AstNode::Number { number: 5. })
                },
                &map
            ),
            Ok(NumberOrMatrix::Number(16.))
        );

        assert_eq!(
            AstNode::evaluate(
                AstNode::Add {
                    left: Box::new(AstNode::Number { number: 1. }),
                    right: Box::new(AstNode::Number { number: 2. })
                },
                &map
            ),
            Ok(NumberOrMatrix::Number(3.))
        );
    }
}
