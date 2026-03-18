use bevy::prelude::*;

mod combat;
mod dialogue;
mod level;
mod menu;
mod player;
mod state;
mod sword;

use combat::CombatPlugin;
use dialogue::DialoguePlugin;
use level::LevelPlugin;
use menu::MenuPlugin;
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
        .add_plugins((
            MenuPlugin,
            DialoguePlugin,
            PlayerPlugin,
            SwordPlugin,
            CombatPlugin,
            LevelPlugin,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
