//! This module handles the internals of the matrices. Storing, handling, parsing, evaluating, etc.

use glam::f64::{DMat2, DMat3};
use std::{fmt, ops::Mul};
use thiserror::Error;

pub mod expression;
pub mod map;

/// The name of a named matrix. Essentially a variable name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatrixName<'n>(&'n str);

/// A 2D or 3D matrix.
#[derive(Clone, Debug, PartialEq)]
pub enum Matrix2dOr3d {
    TwoD(DMat2),
    ThreeD(DMat3),
}

impl From<DMat2> for Matrix2dOr3d {
    fn from(value: DMat2) -> Self {
        Self::TwoD(value)
    }
}

impl From<DMat3> for Matrix2dOr3d {
    fn from(value: DMat3) -> Self {
        Self::ThreeD(value)
    }
}

impl Mul<Matrix2dOr3d> for f64 {
    type Output = Matrix2dOr3d;

    fn mul(self, rhs: Matrix2dOr3d) -> Self::Output {
        match rhs {
            Matrix2dOr3d::TwoD(matrix) => Matrix2dOr3d::TwoD(self * matrix),
            Matrix2dOr3d::ThreeD(matrix) => Matrix2dOr3d::ThreeD(self * matrix),
        }
    }
}

impl Mul<f64> for Matrix2dOr3d {
    type Output = Matrix2dOr3d;

    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            Matrix2dOr3d::TwoD(matrix) => Matrix2dOr3d::TwoD(matrix * rhs),
            Matrix2dOr3d::ThreeD(matrix) => Matrix2dOr3d::ThreeD(matrix * rhs),
        }
    }
}

/// Cannot multiply two matrices of different dimensions.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub struct CannotMultiplyDifferentDimensions;

impl fmt::Display for CannotMultiplyDifferentDimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot multiply two matrices of different dimensions")
    }
}

/// Cannot add two matrices of different dimensions.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub struct CannotAddDifferentDimensions;

impl fmt::Display for CannotAddDifferentDimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot add two matrices of different dimensions")
    }
}

impl Matrix2dOr3d {
    /// Try to multiply two matrices together.
    ///
    /// This method will fail if the two matrices are of different dimensions.
    pub fn try_mul(left: Self, right: Self) -> Result<Self, CannotMultiplyDifferentDimensions> {
        match (left, right) {
            (Self::TwoD(a), Self::TwoD(b)) => Ok(Self::TwoD(a * b)),
            (Self::ThreeD(a), Self::ThreeD(b)) => Ok(Self::ThreeD(a * b)),
            _ => Err(CannotMultiplyDifferentDimensions),
        }
    }

    /// Try to add two matrices together.
    ///
    /// This method will fail if the two matrices are of different dimensions.
    pub fn try_add(left: Self, right: Self) -> Result<Self, CannotAddDifferentDimensions> {
        match (left, right) {
            (Self::TwoD(a), Self::TwoD(b)) => Ok(Self::TwoD(a + b)),
            (Self::ThreeD(a), Self::ThreeD(b)) => Ok(Self::ThreeD(a + b)),
            _ => Err(CannotAddDifferentDimensions),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::{AbsDiffEq, RelativeEq};

    impl AbsDiffEq for Matrix2dOr3d {
        type Epsilon = <f64 as AbsDiffEq>::Epsilon;

        fn default_epsilon() -> Self::Epsilon {
            <f64 as AbsDiffEq>::default_epsilon()
        }

        fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
            match (self, other) {
                (Self::TwoD(a), Self::TwoD(b)) => a.abs_diff_eq(*b, epsilon),
                (Self::ThreeD(a), Self::ThreeD(b)) => a.abs_diff_eq(*b, epsilon),
                _ => false,
            }
        }
    }

    impl RelativeEq for Matrix2dOr3d {
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
                (Self::TwoD(a), Self::TwoD(b)) => a.relative_eq(b, epsilon, max_relative),
                (Self::ThreeD(a), Self::ThreeD(b)) => a.relative_eq(b, epsilon, max_relative),
                _ => false,
            }
        }
    }
}
