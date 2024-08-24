//! This module handles and provides the [`MatrixMap`] trait and its primary implementors,
//! [`MatrixMap2`] and [`MatrixMap3`].

use super::{Matrix2dOr3d, MatrixName};
use glam::{DMat2, DMat3};
use std::collections::HashMap;
use thiserror::Error;

/// All the stuff you want from this module.
pub mod prelude {
    pub use super::{MatrixMap, MatrixMap2, MatrixMap3, MatrixMapError};
}

/// An error which can be returned by a method of [`MatrixMap`].
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum MatrixMapError {
    /// The matrix has an invalide name. See [`MatrixName`].
    #[error("Invalid name for matrix: \"{0}\"")]
    InvalidName(String),

    /// The matrix with this name is not defined in the map.
    #[error("Matrix named \"{0}\" is not defined")]
    NameNotDefined(String),
}

/// A map from names to defined matrices.
pub trait MatrixMap {
    /// The type of matrix that this map holds.
    type MatrixType: Into<Matrix2dOr3d>;

    /// Create a new, empty matrix map.
    fn new() -> Self;

    /// Set the value of the matrix with the given name.
    ///
    /// This method will blindly overwrite the old value if a matrix with this name already exists.
    /// Use [`MatrixMap::get`] first to check.
    fn set(&mut self, name: MatrixName<'_>, value: Self::MatrixType) -> Result<(), MatrixMapError>;

    /// Get the named matrix from the map, if it exists.
    fn get(&self, name: MatrixName<'_>) -> Result<Self::MatrixType, MatrixMapError>;
}

/// A [`MatrixMap`] for 2D matrices
#[derive(Clone, Debug, PartialEq)]
pub struct MatrixMap2 {
    /// The [`HashMap`] of [`DMat2`] backing this implementation.
    map: HashMap<String, DMat2>,
}

impl MatrixMap for MatrixMap2 {
    type MatrixType = DMat2;

    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn set(
        &mut self,
        MatrixName(name): MatrixName,
        value: Self::MatrixType,
    ) -> Result<(), MatrixMapError> {
        self.map.insert(name.to_owned(), value);
        Ok(())
    }

    fn get(&self, MatrixName(name): MatrixName) -> Result<Self::MatrixType, MatrixMapError> {
        match self.map.get(name) {
            Some(matrix) => Ok(*matrix),
            None => Err(MatrixMapError::NameNotDefined(name.to_owned())),
        }
    }
}

// NOTE: The impl for 3D matrices is identical to the impl for 2D ones and should always remain
// identical. I might pull this out into a derive macro if it becomes complex enough.

/// A [`MatrixMap`] for 3D matrices
#[derive(Clone, Debug, PartialEq)]
pub struct MatrixMap3 {
    /// The [`HashMap`] of [`DMat3`] backing this implementation.
    map: HashMap<String, DMat3>,
}

impl MatrixMap for MatrixMap3 {
    type MatrixType = DMat3;

    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn set(
        &mut self,
        MatrixName(name): MatrixName,
        value: Self::MatrixType,
    ) -> Result<(), MatrixMapError> {
        self.map.insert(name.into(), value);
        Ok(())
    }

    fn get(&self, MatrixName(name): MatrixName) -> Result<Self::MatrixType, MatrixMapError> {
        match self.map.get(name) {
            Some(matrix) => Ok(*matrix),
            None => Err(MatrixMapError::NameNotDefined(name.to_owned())),
        }
    }
}
