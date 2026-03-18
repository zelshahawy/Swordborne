use bevy::prelude::*;

use crate::dialogue::gameplay_unlocked;
use crate::state::GameState;

pub mod slash;

use slash::{start_slash, tick_player_action};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (start_slash, tick_player_action)
                .chain()
                .run_if(in_state(GameState::InGame))
                .run_if(gameplay_unlocked),
        );
    }
}
