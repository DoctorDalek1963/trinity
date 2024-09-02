//! This module provides [`start_bevy`], the entrypoint for Trinity's Bevy runtime.

use bevy::{prelude::*, winit::WinitSettings};

/// Start Bevy and setup everything needed for Trinity's graphics.
pub fn start_bevy() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins).add_systems(Startup, setup);

    #[cfg(not(target_family = "wasm"))]
    {
        // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
        app.insert_resource(WinitSettings::desktop_app());
    }

    #[cfg(feature = "dev_tools")]
    {
        app.add_plugins(bevy::dev_tools::ui_debug_overlay::DebugUiPlugin)
            .add_systems(Update, toggle_overlay);
    }

    app.run();
}

/// Setup everything we need for Bevy.
fn setup(
    mut commands: Commands,
    // asset_server: Res<AssetServer>
) {
    commands.spawn((Camera2dBundle::default(), IsDefaultUiCamera));
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
