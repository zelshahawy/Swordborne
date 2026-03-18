use bevy::prelude::*;

pub mod components;
pub mod logic;
pub mod spawn;

use logic::{pickup_sword, throw_sword, update_flying_sword};
use spawn::{load_sword_visuals, spawn_sword_at_start};

pub struct SwordPlugin;

impl Plugin for SwordPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_sword_visuals, spawn_sword_at_start).chain())
            .add_systems(Update, (pickup_sword, throw_sword, update_flying_sword).chain());
    }
}

pub use components::*;
