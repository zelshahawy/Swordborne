use bevy::prelude::*;

use crate::dialogue::gameplay_unlocked;
use crate::state::GameState;

pub mod aim;
pub mod components;
pub mod logic;
pub mod spawn;

use aim::{
    begin_sword_aim, despawn_aim_preview, release_sword_aim, spawn_aim_preview,
    update_sword_aim_preview,
};
use logic::{pickup_sword, update_flying_sword, update_sword_trail};
use spawn::load_sword_visuals;

pub struct SwordPlugin;

impl Plugin for SwordPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<components::SwordAimState>()
            .add_systems(Startup, load_sword_visuals)
            .add_systems(OnEnter(GameState::InGame), spawn_aim_preview)
            .add_systems(OnExit(GameState::InGame), despawn_aim_preview)
            .add_systems(
                Update,
                (
                    pickup_sword,
                    begin_sword_aim,
                    update_sword_aim_preview,
                    release_sword_aim,
                    update_flying_sword,
                    update_sword_trail,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame))
                    .run_if(gameplay_unlocked),
            );
    }
}

pub use components::*;
