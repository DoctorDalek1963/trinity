//! This module provides [`start_bevy`], the entrypoint for Trinity's Bevy runtime.

use super::{IsBasisVectorI, IsBasisVectorJ};
use bevy::prelude::*;

/// Start Bevy and setup everything needed for Trinity's graphics.
pub fn run_bevy() -> AppExit {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: String::from("Trinity"),

            // Wasm
            canvas: Some(String::from("#trinity-bevy")),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: true,

            ..default()
        }),
        ..default()
    }))
    .add_systems(Startup, setup);

    #[cfg(feature = "dev_tools")]
    {
        app.add_plugins(bevy::dev_tools::ui_debug_overlay::DebugUiPlugin)
            .add_systems(Update, toggle_overlay);
    }

    app.run()
}

/// Setup everything we need for Bevy.
fn setup(mut commands: Commands) {
    // Spawn the camera. We set the projection.scale to 1/100 so that all the entities can use the
    // actual values given by the matrices and whatnot, so that we don't have to scale up and down
    // between calculating matrix stuff and actually rendering it.
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            near: -10.,
            far: 10.,
            scale: 1. / 100.,
            ..default()
        },
        ..default()
    });

    // Spawn the i and j basis vectors
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: bevy::color::palettes::tailwind::BLUE_500.into(),
                custom_size: Some(Vec2::new(0.1, 0.1)),
                ..default()
            },
            transform: Transform::from_xyz(1., 0., 0.),
            ..default()
        },
        IsBasisVectorI,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: bevy::color::palettes::tailwind::RED_500.into(),
                custom_size: Some(Vec2::new(0.1, 0.1)),
                ..default()
            },
            transform: Transform::from_xyz(0., 1., 0.),
            ..default()
        },
        IsBasisVectorJ,
    ));
}

/// Toggle the debug outlines around nodes.
#[cfg(feature = "dev_tools")]
fn toggle_overlay(
    input: Res<ButtonInput<KeyCode>>,
    mut options: ResMut<bevy::dev_tools::ui_debug_overlay::UiDebugOptions>,
) {
    if input.just_pressed(KeyCode::Space) {
        options.toggle();
    }
}
