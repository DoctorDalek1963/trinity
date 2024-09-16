//! This module handles everything to do with Bevy, the game engine used by Trinity.

mod entrypoint;

pub use self::entrypoint::start_bevy;

use bevy::prelude::Component;

/// This entity is the basis vector `i`.
#[derive(Component)]
struct IsBasisVectorI;

/// This entity is the basis vector `j`.
#[derive(Component)]
struct IsBasisVectorJ;
