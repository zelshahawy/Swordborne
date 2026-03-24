use bevy::prelude::*;

mod combat;
mod dialogue;
mod fonts;
mod level;
mod menu;
mod player;
mod puzzle;
mod state;
mod sword;

use combat::CombatPlugin;
use dialogue::DialoguePlugin;
use fonts::FontPlugin;
use level::LevelPlugin;
use menu::MenuPlugin;
use player::PlayerPlugin;
use puzzle::PuzzlePlugin;
use state::GameState;
use sword::SwordPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SwordBorne".into(),
                resolution: (1600, 900).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_plugins((
            FontPlugin,
            MenuPlugin,
            DialoguePlugin,
            PlayerPlugin,
            SwordPlugin,
            CombatPlugin,
            LevelPlugin,
            PuzzlePlugin,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
