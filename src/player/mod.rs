use bevy::prelude::*;

use crate::dialogue::gameplay_unlocked;
use crate::state::GameState;

pub mod animation;
pub mod collision;
pub mod components;
pub mod movement;
pub mod spawn;

use animation::{animate_player, select_animation, update_player_flip};
use collision::move_player;
use movement::{apply_gravity, player_input};
use spawn::load_player_animations;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_player_animations)
            .add_systems(
                Update,
                (
                    player_input,
                    apply_gravity,
                    move_player,
                    select_animation,
                    animate_player,
                    update_player_flip,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame))
                    .run_if(gameplay_unlocked),
            );
    }
}

pub use components::*;
