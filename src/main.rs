use bevy::prelude::*;

mod combat;
mod player;
mod puzzle;
mod state;
mod sword;

use combat::CombatPlugin;
use player::PlayerPlugin;
use puzzle::PuzzlePlugin;
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
        .add_plugins((PlayerPlugin, SwordPlugin, CombatPlugin, PuzzlePlugin))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
