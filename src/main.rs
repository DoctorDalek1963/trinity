//! Trinity is a program built to visualise and interact with matrices in the form of linear
//! transformations.

#![warn(missing_docs, clippy::missing_docs_in_private_items)]

pub mod bevy;
pub mod math;
pub mod matrix;

fn main() -> ::bevy::prelude::AppExit {
    self::bevy::run_bevy()
}
