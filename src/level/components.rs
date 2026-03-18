use bevy::prelude::*;

use crate::level::{
    ROOM_CEILING_Y, ROOM_PLAYER_LEFT_X, ROOM_PLAYER_RIGHT_X, ROOM_WALL_LEFT_X, ROOM_WALL_RIGHT_X,
};
use crate::state::LevelId;

#[derive(Resource, Debug, Clone, Copy)]
pub struct LevelBounds {
    pub wall_left_x: f32,
    pub wall_right_x: f32,
    pub player_left_x: f32,
    pub player_right_x: f32,
    pub ceiling_y: f32,
}

impl Default for LevelBounds {
    fn default() -> Self {
        Self {
            wall_left_x: ROOM_WALL_LEFT_X,
            wall_right_x: ROOM_WALL_RIGHT_X,
            player_left_x: ROOM_PLAYER_LEFT_X,
            player_right_x: ROOM_PLAYER_RIGHT_X,
            ceiling_y: ROOM_CEILING_Y,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct PendingLevelTransition {
    pub next_level: Option<LevelId>,
}

#[derive(Component)]
pub(crate) struct LevelEntity;

#[derive(Component)]
pub(crate) struct WizardNpc;

#[derive(Component)]
pub(crate) struct WizardAnimationTimer(pub Timer);

#[derive(Component, Default)]
pub(crate) struct WizardAnimationFrame(pub usize);

#[derive(Component)]
pub(crate) struct TutorialMarker;

#[derive(Component)]
pub(crate) struct BreakableCrate {
    pub reward: CrateReward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CrateReward {
    OpenTrainingDoor,
    CompleteLevelTwo,
}

#[derive(Component)]
pub(crate) struct CrateBreakShard {
    pub velocity: Vec2,
    pub spin_speed: f32,
    pub timer: Timer,
}

#[derive(Component)]
pub(crate) struct TrainingDoor {
    pub open: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct SwordBlocker {
    pub half_extents: Vec2,
}

#[derive(Component)]
pub(crate) struct LevelTwoCompletionText;
