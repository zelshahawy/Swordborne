use bevy::prelude::*;

mod player;
mod state;
mod sword;

use player::PlayerPlugin;
use state::GameState;
use sword::SwordPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SwordBorne".into(),
                resolution: (1280, 720).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_plugins((PlayerPlugin, SwordPlugin))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
