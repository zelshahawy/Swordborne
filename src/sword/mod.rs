use bevy::prelude::*;

use crate::dialogue::gameplay_unlocked;
use crate::state::GameState;

pub mod components;
pub mod logic;
pub mod spawn;

use logic::{pickup_sword, throw_sword, update_flying_sword, update_sword_trail};
use spawn::load_sword_visuals;

pub struct SwordPlugin;

impl Plugin for SwordPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sword_visuals).add_systems(
            Update,
            (
                pickup_sword,
                throw_sword,
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
