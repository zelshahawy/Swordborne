use bevy::prelude::*;

use crate::dialogue::gameplay_unlocked;
use crate::state::GameState;

mod assets;
mod components;
mod logic;
mod spawn;

use assets::load_level_art;
use logic::{
    animate_wizard_idle, apply_level_transition, break_training_crate, constrain_player_to_level,
    trigger_tutorial_hint, trigger_wizard_followup, trigger_wizard_intro, try_advance_level,
    update_training_door_visual,
};
use spawn::{despawn_level_entities, spawn_current_level};

pub use components::LevelBounds;

pub struct LevelPlugin;

pub(super) const TILE_SCALE: f32 = 4.0;
pub(super) const TILE_WORLD_SIZE: f32 = 64.0;
pub(super) const ROOM_WALL_LEFT_X: f32 = -600.0;
pub(super) const ROOM_WALL_RIGHT_X: f32 = 600.0;
pub(super) const ROOM_PLAYER_LEFT_X: f32 = -560.0;
pub(super) const ROOM_PLAYER_RIGHT_X: f32 = 600.0;
pub(super) const LEVEL_ONE_PLAYER_START_X: f32 = -520.0;
pub(super) const LEVEL_ONE_DOOR_X: f32 = 540.0;
pub(super) const LEVEL_ONE_WIZARD_X: f32 = -280.0;
pub(super) const LEVEL_ONE_SWORD_X: f32 = 40.0;
pub(super) const LEVEL_ONE_TUTORIAL_X: f32 = 120.0;
pub(super) const LEVEL_ONE_CRATE_X: f32 = 330.0;

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
                apply_level_transition.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (
                    constrain_player_to_level,
                    trigger_wizard_intro,
                    trigger_wizard_followup,
                    trigger_tutorial_hint,
                    break_training_crate,
                    update_training_door_visual,
                    try_advance_level,
                )
                    .run_if(in_state(GameState::InGame))
                    .run_if(gameplay_unlocked),
            );
    }
}
