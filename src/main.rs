use bevy::prelude::*;

mod boss;
mod combat;
mod dialogue;
mod fonts;
mod level;
mod menu;
mod pause;
mod player;
mod puzzle;
mod state;
mod sword;
mod victory;

#[cfg(not(target_arch = "wasm32"))]
mod leaderboard;

use boss::BossPlugin;
use combat::CombatPlugin;
use dialogue::DialoguePlugin;
use fonts::FontPlugin;
use level::LevelPlugin;
use menu::MenuPlugin;
use pause::PausePlugin;
use player::PlayerPlugin;
use puzzle::PuzzlePlugin;
use state::GameState;
use sword::SwordPlugin;
use victory::VictoryPlugin;

#[cfg(not(target_arch = "wasm32"))]
use leaderboard::LeaderboardPlugin;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = dotenvy::dotenv();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "SwordBorne".into(),
            resolution: (1600, 900).into(),
            resizable: false,
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#bevy-container".to_string()),
            #[cfg(target_arch = "wasm32")]
            fit_canvas_to_parent: true,
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
        BossPlugin,
        PausePlugin,
        VictoryPlugin,
    ));

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(LeaderboardPlugin);

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
