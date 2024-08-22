//! This module handles and provides [`MatrixMap`].

use super::Matrix2dOr3d;
use std::collections::HashMap;
use thiserror::Error;

/// A map from names to defined matrices.
#[derive(Clone, Debug, PartialEq)]
pub struct MatrixMap {
    map: HashMap<String, Matrix2dOr3d>,
}

impl Default for MatrixMap {
    fn default() -> Self {
        Self::new()
    }
}

impl MatrixMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

/// An error which can be returned by a method of [`MatrixMap`].
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum MatrixMapError {
    #[error("Invalid name for matrix: \"{0}\"")]
    InvalidName(String),

    #[error("Matrix named \"{0}\" is not defined")]
    NameNotDefined(String),
}

impl MatrixMap {
    /// Set the value of the matrix with the given name.
    ///
    /// This method will blindly overwrite the old value if a matrix with this name already exists.
    /// Use [`MatrixMap::get`] first to check.
    pub fn set(
        &mut self,
        name: impl Into<String>,
        value: Matrix2dOr3d,
    ) -> Result<(), MatrixMapError> {
        // TODO: Validate name
        self.map.insert(name.into(), value);
        Ok(())
    }

    /// Get the named matrix from the map, if it exists.
    pub fn get(&self, name: impl Into<String>) -> Result<Matrix2dOr3d, MatrixMapError> {
        // TODO: Validate name
        let name: String = name.into();
        match self.map.get(&name) {
            Some(matrix) => Ok(matrix.clone()),
            None => Err(MatrixMapError::NameNotDefined(name)),
        }
    }
}
