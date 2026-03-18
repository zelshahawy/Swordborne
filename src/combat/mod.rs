use bevy::prelude::*;

pub mod slash;

use slash::{start_slash, tick_player_action};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (start_slash, tick_player_action).chain());
    }
}
