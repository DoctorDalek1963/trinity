//! This module handles and provides the [`MatrixMap`] trait and its primary implementors,
//! [`MatrixMap2`] and [`MatrixMap3`].

use glam::{DMat2, DMat3};
use std::collections::HashMap;
use thiserror::Error;

pub mod prelude {
    pub use super::{MatrixMap, MatrixMap2, MatrixMap3, MatrixMapError};
}

/// An error which can be returned by a method of [`MatrixMap`].
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum MatrixMapError {
    #[error("Invalid name for matrix: \"{0}\"")]
    InvalidName(String),

    #[error("Matrix named \"{0}\" is not defined")]
    NameNotDefined(String),
}

/// A map from names to defined matrices.
pub trait MatrixMap {
    type MatrixType;

    fn new() -> Self;

    fn set(
        &mut self,
        name: impl Into<String>,
        value: Self::MatrixType,
    ) -> Result<(), MatrixMapError>;

    fn get(&self, name: impl Into<String>) -> Result<Self::MatrixType, MatrixMapError>;
}

/// A [`MatrixMap`] for 2D matrices
#[derive(Clone, Debug, PartialEq)]
pub struct MatrixMap2 {
    map: HashMap<String, DMat2>,
}

impl MatrixMap for MatrixMap2 {
    type MatrixType = DMat2;

    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Set the value of the matrix with the given name.
    ///
    /// This method will blindly overwrite the old value if a matrix with this name already exists.
    /// Use [`MatrixMap::get`] first to check.
    fn set(
        &mut self,
        name: impl Into<String>,
        value: Self::MatrixType,
    ) -> Result<(), MatrixMapError> {
        // TODO: Validate name
        self.map.insert(name.into(), value);
        Ok(())
    }

    /// Get the named matrix from the map, if it exists.
    fn get(&self, name: impl Into<String>) -> Result<Self::MatrixType, MatrixMapError> {
        // TODO: Validate name
        let name: String = name.into();
        match self.map.get(&name) {
            Some(matrix) => Ok(*matrix),
            None => Err(MatrixMapError::NameNotDefined(name)),
        }
    }
}

// NOTE: The impl for 3D matrices is identical to the impl for 2D ones and should always remain
// identical. I might pull this out into a derive macro if it becomes complex enough.

/// A [`MatrixMap`] for 3D matrices
#[derive(Clone, Debug, PartialEq)]
pub struct MatrixMap3 {
    map: HashMap<String, DMat3>,
}

impl MatrixMap for MatrixMap3 {
    type MatrixType = DMat3;

    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Set the value of the matrix with the given name.
    ///
    /// This method will blindly overwrite the old value if a matrix with this name already exists.
    /// Use [`MatrixMap::get`] first to check.
    fn set(
        &mut self,
        name: impl Into<String>,
        value: Self::MatrixType,
    ) -> Result<(), MatrixMapError> {
        // TODO: Validate name
        self.map.insert(name.into(), value);
        Ok(())
    }

    /// Get the named matrix from the map, if it exists.
    fn get(&self, name: impl Into<String>) -> Result<Self::MatrixType, MatrixMapError> {
        // TODO: Validate name
        let name: String = name.into();
        match self.map.get(&name) {
            Some(matrix) => Ok(*matrix),
            None => Err(MatrixMapError::NameNotDefined(name)),
        }
    }
}
