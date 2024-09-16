//! This module handles everything to do with Bevy, the game engine used by Trinity.

mod setup;

pub use self::setup::run_bevy;

use bevy::prelude::Component;

/// This entity is the basis vector `i`.
#[derive(Component)]
struct IsBasisVectorI;

/// This entity is the basis vector `j`.
#[derive(Component)]
struct IsBasisVectorJ;
