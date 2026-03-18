use bevy::prelude::*;

use crate::state::LevelId;

use super::{
    ROOM_CEILING_Y, ROOM_PLAYER_LEFT_X, ROOM_PLAYER_RIGHT_X, ROOM_WALL_LEFT_X, ROOM_WALL_RIGHT_X,
};

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
pub(super) struct PendingLevelTransition {
    pub next_level: Option<LevelId>,
}

#[derive(Component)]
pub(super) struct LevelEntity;

#[derive(Component)]
pub(super) struct WizardNpc;

#[derive(Component)]
pub(super) struct WizardAnimationTimer(pub Timer);

#[derive(Component, Default)]
pub(super) struct WizardAnimationFrame(pub usize);

#[derive(Component)]
pub(super) struct TutorialMarker;

#[derive(Component)]
pub(super) struct TrainingCrate;

#[derive(Component)]
pub(super) struct TrainingDoor {
    pub open: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct SwordBlocker {
    pub half_extents: Vec2,
}

#[derive(Component)]
pub(super) struct TrialChest {
    pub open: bool,
}

#[derive(Component)]
pub(super) struct LevelTwoCompletionText;
