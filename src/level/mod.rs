use bevy::prelude::*;

use crate::dialogue::gameplay_unlocked;
use crate::player::GROUND_Y;
use crate::state::GameState;

mod assets;
mod components;
mod logic;
mod scene;
mod spawn;

pub(crate) use assets::{LevelArtHandles, load_level_art};
pub(crate) use components::{
    BreakableChest, BreakableCrate, CrateBreakShard, CrateReward, LevelBounds, LevelEntity,
    LevelFourCompletionText, LevelThreeCompletionText, LevelTwoCompletionText,
    PendingLevelTransition, SwordBlocker, TrainingDoor, WizardAnimationFrame, WizardAnimationTimer,
    WizardNpc,
};
pub(crate) use logic::{
    animate_wizard_idle, apply_level_transition, break_crates, constrain_player_to_level, open_chests,
    execute_level_restart, request_level_restart, sync_level_four_completion_text,
    sync_level_three_completion_text, sync_level_two_completion_text, sync_level_two_door,
    trigger_dark_wizard_intro, trigger_wizard_followup, trigger_wizard_intro, try_advance_level,
    update_crate_break_shards, update_training_door_visual,
};
pub(crate) use scene::{
    frame_level_camera, spawn_bottom_anchored_sprite, spawn_centered_tile, spawn_room_shell,
    update_level_camera,
};
pub(crate) use spawn::{
    despawn_level_entities, level_bounds_for, level_camera_focus_x, spawn_current_level,
    spawn_level_scene,
};

pub struct LevelPlugin;

pub(crate) const TILE_SCALE: f32 = 4.0;
pub(crate) const WIZARD_SCALE: f32 = 4.0;
pub(crate) const TILE_WORLD_SIZE: f32 = 64.0;
pub(crate) const ROOM_TILE_COLUMNS: usize = 24;
pub(crate) const ROOM_WALL_ROWS: usize = 10;
pub(crate) const ROOM_WALL_LEFT_X: f32 = -768.0;
pub(crate) const ROOM_WALL_RIGHT_X: f32 = 768.0;
pub(crate) const ROOM_PLAYER_LEFT_X: f32 = -720.0;
pub(crate) const ROOM_PLAYER_RIGHT_X: f32 = 720.0;
pub(crate) const ROOM_CEILING_Y: f32 = GROUND_Y + 300.0;
pub(crate) const LEVEL_ONE_PLAYER_START_X: f32 = -676.0;
pub(crate) const LEVEL_ONE_DOOR_X: f32 = 680.0;
pub(crate) const LEVEL_ONE_WIZARD_X: f32 = -352.0;
pub(crate) const LEVEL_ONE_SWORD_X: f32 = 28.0;
pub(crate) const LEVEL_ONE_TUTORIAL_X: f32 = 164.0;
pub(crate) const LEVEL_ONE_CRATE_X: f32 = 470.0;
pub(crate) const LEVEL_TWO_PLAYER_START_X: f32 = 0.0;
pub(crate) const LEVEL_TWO_CRATE_X: f32 = 0.0;
pub(crate) const LEVEL_TWO_SHELF_TOP_Y: f32 = GROUND_Y + 208.0;
pub(crate) const LEVEL_TWO_HINT_X: f32 = 0.0;
pub(crate) const LEVEL_TWO_HINT_Y: f32 = GROUND_Y + 118.0;
pub(crate) const LEVEL_TWO_DOOR_X: f32 = 650.0;

// Level 3 – sequence puzzle
pub(crate) const LEVEL_THREE_PLAYER_START_X: f32 = -620.0;
pub(crate) const LEVEL_THREE_DOOR_X: f32 = 650.0;
// Block positions (sequence shown on-screen is Green → Red → Blue)
pub(crate) const LEVEL_THREE_GREEN_X: f32 = 300.0;
pub(crate) const LEVEL_THREE_RED_X: f32 = -100.0;
pub(crate) const LEVEL_THREE_BLUE_X: f32 = -450.0;

pub(crate) const LEVEL_FOUR_PLAYER_START_X: f32 = -620.0;
pub(crate) const LEVEL_FOUR_DOOR_X: f32 = 650.0;
pub(crate) const LEVEL_FOUR_BLUE_A_X: f32 = -530.0; // ground, far left
pub(crate) const LEVEL_FOUR_BLUE_B_X: f32 = 530.0; // ground, far right
pub(crate) const LEVEL_FOUR_RED_A_X: f32 = -240.0; // elevated left platform
pub(crate) const LEVEL_FOUR_GREEN_X: f32 = 0.0; // elevated center platform
pub(crate) const LEVEL_FOUR_RED_B_X: f32 = 240.0; // elevated right platform
pub(crate) const LEVEL_FIVE_PLAYER_START_X: f32 = -580.0;
pub(crate) const LEVEL_FIVE_BOSS_START_X: f32 = 380.0;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<components::LevelBounds>()
            .init_resource::<components::PendingLevelTransition>()
            .add_systems(Startup, load_level_art)
            .add_systems(OnEnter(GameState::InGame), spawn_current_level)
            .add_systems(OnExit(GameState::InGame), despawn_level_entities)
            .add_systems(
                Update,
                animate_wizard_idle.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                update_crate_break_shards.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                update_level_camera
                    .run_if(in_state(GameState::InGame))
                    .run_if(gameplay_unlocked),
            )
            .add_systems(
                Update,
                apply_level_transition
                    .after(try_advance_level)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (request_level_restart, execute_level_restart).run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (
                    constrain_player_to_level,
                    trigger_wizard_intro,
                    trigger_wizard_followup,
                    trigger_dark_wizard_intro,
                    break_crates,
                    open_chests,
                    update_training_door_visual,
                    sync_level_two_door,
                    sync_level_two_completion_text,
                    sync_level_three_completion_text,
                    sync_level_four_completion_text,
                    try_advance_level,
                )
                    .run_if(in_state(GameState::InGame))
                    .run_if(gameplay_unlocked),
            );
    }
}
