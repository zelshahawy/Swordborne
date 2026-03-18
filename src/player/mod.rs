use bevy::prelude::*;

pub mod animation;
pub mod components;
pub mod movement;
pub mod spawn;

use animation::{animate_player, select_animation, update_player_flip};
use movement::{apply_gravity, move_player, player_input};
use spawn::{load_player_animations, spawn_player};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_player_animations, spawn_player).chain())
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
                    .chain(),
            );
    }
}

pub use components::*;
