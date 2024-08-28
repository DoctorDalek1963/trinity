//! This module handles the internals of the matrices. Storing, handling, parsing, evaluating, etc.

use glam::f64::{DMat2, DMat3};
use lazy_static::lazy_static;
use regex::Regex;
use std::{fmt, ops::Mul};
use thiserror::Error;

pub mod expression;
pub mod map;

/// The string used to build [`LEADING_MATRIX_NAME_REGEX`] and [`FULL_MATRIX_NAME_REGEX`].
const REGEX_STRING: &str = r"^[A-Z][A-Za-z0-9_]*";

lazy_static! {
    /// Matches a valid matrix name at the start of the string.
    pub static ref LEADING_MATRIX_NAME_REGEX: Regex = Regex::new(REGEX_STRING).unwrap();

    /// Matches a valid matrix name which takes up the whole string.
    pub static ref FULL_MATRIX_NAME_REGEX: Regex = Regex::new(&format!("{REGEX_STRING}$")).unwrap();
}

/// The name of a named matrix. Essentially a variable name.
///
/// A matrix name must start with an uppercase letter, and can contain letters, numbers, and
/// underscores. `M`, `Matrix`, `X_2`, and `M3_Y5F` are all valid matrix names. But `m`, `matrix`,
/// `X-2`, and `M5#Y` are all invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatrixName<'n> {
    /// The name of the matrix. Should be pre-validated by [`MatrixName::new`].
    name: &'n str,
}

impl<'n> MatrixName<'n> {
    /// Create a new matrix name.
    ///
    /// In debug builds, this function will panic if the name is invalid (see [`Self::is_valid`]).
    /// In non-debug builds, this function will never panic, since the only code paths that should
    /// ever call [`MatrixName::new`] should only pass names that are already known to be valid.
    pub fn new(name: &'n str) -> Self {
        debug_assert!(Self::is_valid(name), "MatrixName must be valid");
        Self { name }
    }

    /// Check if the matrix name is valid. See the [`MatrixName`] docs for valid names.
    pub fn is_valid(name: &str) -> bool {
        FULL_MATRIX_NAME_REGEX.is_match(name)
    }
}

/// A 2D or 3D matrix.
#[derive(Clone, Debug, PartialEq)]
pub enum Matrix2dOr3d {
    /// A two dimensional matrix.
    TwoD(DMat2),

    /// A three dimensional matrix.
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

#[mutants::skip]
impl fmt::Display for CannotMultiplyDifferentDimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot multiply two matrices of different dimensions")
    }
}

/// Cannot add two matrices of different dimensions.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub struct CannotAddDifferentDimensions;

#[mutants::skip]
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

    #[test]
    fn matrix_name_is_valid() {
        let valid_names = [
            "M",
            "Mat",
            "A2",
            "X_Y3",
            "Dave",
            "N",
            "T",
            "SomeReallyLongMatrixNameButIts_Okay_Because_It_fits_all_the_rules",
        ];
        for name in valid_names {
            assert!(MatrixName::is_valid(name), "'{name}' should be valid");
        }

        let invalid_names = [
            "",
            "m",
            " M",
            "x",
            "my_matrix",
            "::",
            "Name with spaces",
            "WhatAboutPunctuation?",
            "It's",
            "X:C",
        ];
        for name in invalid_names {
            assert!(!MatrixName::is_valid(name), "'{name}' should be invalid");
        }
    }

    // Should panic iff we're in a debug build
    #[test]
    #[cfg_attr(debug_assertions, should_panic = "MatrixName must be valid")]
    fn matrix_name_new_panics() {
        MatrixName::new("m");
    }
}
