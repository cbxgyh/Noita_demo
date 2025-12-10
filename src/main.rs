mod atoms;
mod physics;
mod rendering;
mod game;
mod magic;
mod level_generation;
mod level_editor;
mod sound;
mod touchscreen;
mod networking;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Noita Demo - Falling Sand Game".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        // 32 pixels = 1 physics meter for Rapier (2D)
        .add_plugins(
            bevy_rapier2d::prelude::RapierPhysicsPlugin::<bevy_rapier2d::prelude::NoUserData>::pixels_per_meter(
                32.0,
            ),
        )
        .add_plugins(bevy_rapier2d::prelude::RapierDebugRenderPlugin::default())
        .add_plugins(game::GamePlugin)
        .run();
}
