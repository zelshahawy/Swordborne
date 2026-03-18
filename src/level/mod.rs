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
    BreakableCrate, CrateBreakShard, CrateReward, LevelBounds, LevelEntity, LevelTwoCompletionText,
    PendingLevelTransition, SwordBlocker, TrainingDoor, TutorialMarker, WizardAnimationFrame,
    WizardAnimationTimer, WizardNpc,
};
pub(crate) use logic::{
    animate_wizard_idle, apply_level_transition, break_crates, constrain_player_to_level,
    restart_current_level, sync_level_two_completion_text, trigger_tutorial_hint,
    trigger_wizard_followup, trigger_wizard_intro, try_advance_level, update_crate_break_shards,
    update_training_door_visual, wizard_scale,
};
pub(crate) use scene::{frame_level_camera, spawn_room_shell, update_level_camera};
pub(crate) use spawn::{
    despawn_level_entities, level_bounds_for, level_camera_focus_x, spawn_current_level,
    spawn_level_scene,
};

pub struct LevelPlugin;

pub(crate) const TILE_SCALE: f32 = 4.0;
pub(crate) const TILE_WORLD_SIZE: f32 = 64.0;
pub(crate) const ROOM_TILE_COLUMNS: usize = 24;
pub(crate) const ROOM_WALL_ROWS: usize = 8;
pub(crate) const ROOM_WALL_LEFT_X: f32 = -768.0;
pub(crate) const ROOM_WALL_RIGHT_X: f32 = 768.0;
pub(crate) const ROOM_PLAYER_LEFT_X: f32 = -720.0;
pub(crate) const ROOM_PLAYER_RIGHT_X: f32 = 720.0;
pub(crate) const ROOM_CEILING_Y: f32 = GROUND_Y + 304.0;
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
                apply_level_transition.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (restart_current_level,).run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (
                    constrain_player_to_level,
                    trigger_wizard_intro,
                    trigger_wizard_followup,
                    trigger_tutorial_hint,
                    break_crates,
                    update_training_door_visual,
                    sync_level_two_completion_text,
                    try_advance_level,
                )
                    .run_if(in_state(GameState::InGame))
                    .run_if(gameplay_unlocked),
            );
    }
}
