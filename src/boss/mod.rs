use bevy::prelude::*;

use crate::dialogue::gameplay_unlocked;
use crate::state::{BossDefeated, CampaignState, FadeState, GameState, LevelId, PlayerHealth, RunTimer};

mod components;
mod logic;

pub(crate) use components::*;
pub(crate) use logic::{spawn_boss_hp_bar, spawn_player_hp_ui};

use logic::{
    boss_disarm_player, boss_hit_player, boss_take_damage, handle_boss_defeat, sync_boss_hp_bar,
    sync_player_hp_display, tick_boss, tick_player_invincibility,
};

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerHealth>()
            .init_resource::<FadeState>()
            .init_resource::<RunTimer>()
            .init_resource::<BossDefeated>()
            .add_systems(
                Update,
                (boss_take_damage, boss_disarm_player, boss_hit_player, tick_boss, sync_boss_hp_bar)
                    .chain()
                    .run_if(in_state(GameState::InGame))
                    .run_if(gameplay_unlocked)
                    .run_if(|c: Res<CampaignState>| c.current_level == LevelId::LevelFive),
            )
            .add_systems(
                Update,
                handle_boss_defeat
                    .run_if(in_state(GameState::InGame))
                    .run_if(gameplay_unlocked)
                    .run_if(|c: Res<CampaignState>| c.current_level == LevelId::LevelFive),
            )
            .add_systems(
                Update,
                (tick_player_invincibility, sync_player_hp_display)
                    .run_if(in_state(GameState::InGame))
                    .run_if(|c: Res<CampaignState>| c.current_level == LevelId::LevelFive),
            );
    }
}
