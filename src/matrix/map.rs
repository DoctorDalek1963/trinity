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
    InvalidName(smol_str::SmolStr),

    /// The matrix with this name is not defined in the map.
    #[error("Matrix named \"{0}\" is not defined")]
    NameNotDefined(MatrixName),
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
    fn set(&mut self, name: MatrixName, value: Self::MatrixType) -> Result<(), MatrixMapError>;

    /// Get the named matrix from the map, if it exists.
    fn get(&self, name: &MatrixName) -> Result<Self::MatrixType, MatrixMapError>;
}

/// A [`MatrixMap`] for some generic type `T`.
#[derive(Clone, Debug, PartialEq)]
pub struct MatrixMapHashMap<T: Into<Matrix2dOr3d> + Clone + Copy> {
    /// The [`HashMap`] backing this implementation.
    map: HashMap<MatrixName, T>,
}

/// A [`MatrixMap`] for 2D matrices.
pub type MatrixMap2 = MatrixMapHashMap<DMat2>;

/// A [`MatrixMap`] for 3D matrices.
pub type MatrixMap3 = MatrixMapHashMap<DMat3>;

impl<T: Into<Matrix2dOr3d> + Clone + Copy> MatrixMap for MatrixMapHashMap<T> {
    type MatrixType = T;

    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn set(&mut self, name: MatrixName, value: Self::MatrixType) -> Result<(), MatrixMapError> {
        if name.self_is_valid() {
            self.map.insert(name, value);
            Ok(())
        } else {
            Err(MatrixMapError::InvalidName(name.name))
        }
    }

    fn get(&self, name: &MatrixName) -> Result<Self::MatrixType, MatrixMapError> {
        if MatrixName::is_valid(name.name.as_str()) {
            match self.map.get(name) {
                Some(matrix) => Ok(*matrix),
                None => Err(MatrixMapError::NameNotDefined(name.to_owned())),
            }
        } else {
            Err(MatrixMapError::InvalidName(name.name.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_map_set_get() {
        let mut map2 = MatrixMap2::new();
        let mut map3 = MatrixMap3::new();

        let m1 = rand::random::<DMat2>();
        let m2 = rand::random::<DMat2>();
        let n1 = rand::random::<DMat3>();
        let n2 = rand::random::<DMat3>();

        let m1name = MatrixName::new("M_one");
        let m2name = MatrixName::new("M_two");
        let n1name = MatrixName::new("N_one");
        let n2name = MatrixName::new("N_two");

        assert_eq!(map2.set(m1name.clone(), m1), Ok(()));
        assert_eq!(map2.set(m2name.clone(), m2), Ok(()));
        assert_eq!(map3.set(n1name.clone(), n1), Ok(()));
        assert_eq!(map3.set(n2name.clone(), n2), Ok(()));

        assert_eq!(
            map2.set(MatrixName { name: "m".into() }, m1),
            Err(MatrixMapError::InvalidName("m".into()))
        );
        assert_eq!(
            map3.set(MatrixName { name: "x".into() }, n1),
            Err(MatrixMapError::InvalidName("x".into()))
        );

        assert_eq!(map2.get(&m1name), Ok(m1));
        assert_eq!(map2.get(&m2name), Ok(m2));
        assert_eq!(map3.get(&n1name), Ok(n1));
        assert_eq!(map3.get(&n2name), Ok(n2));

        assert_eq!(
            map2.get(&MatrixName::new("X")),
            Err(MatrixMapError::NameNotDefined(MatrixName::new("X")))
        );
        assert_eq!(
            map2.get(&MatrixName { name: "y".into() }),
            Err(MatrixMapError::InvalidName("y".into()))
        );
        assert_eq!(
            map3.get(&MatrixName::new("X")),
            Err(MatrixMapError::NameNotDefined(MatrixName::new("X")))
        );
        assert_eq!(
            map3.get(&MatrixName { name: "y".into() }),
            Err(MatrixMapError::InvalidName("y".into()))
        );
    }
}
