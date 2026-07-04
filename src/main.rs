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

use leaderboard::LeaderboardPlugin;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = dotenvy::dotenv();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "SwordBorne".into(),
            // 1440x810 fits on 13"/14" MacBook screens (1600 was wider than the
            // display, pushing the top-right HUD off-screen). The camera view at
            // scale 1.2 is still wider than the room, so nothing is cropped.
            resolution: (1440, 810).into(),
            resizable: false,
            // Must point at a real <canvas> element (not a div) — browsers
            // differ on where winit's fallback canvas lands otherwise.
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#bevy-canvas".to_string()),
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

    app.add_plugins(LeaderboardPlugin);

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
