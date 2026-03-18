use bevy::prelude::*;

#[derive(Component)]
pub struct Sword;

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum SwordState {
    Grounded,
    Equipped,
    Flying,
    Dropping,
    Stuck,
}

#[derive(Component, Debug, Default)]
pub struct SwordVelocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct SwordAnimationTimer(pub Timer);

#[derive(Component)]
pub struct SwordFlight {
    pub launch_position: Vec2,
    pub last_trail_position: Vec2,
    pub trail_timer: Timer,
}

impl Default for SwordFlight {
    fn default() -> Self {
        Self {
            launch_position: Vec2::ZERO,
            last_trail_position: Vec2::ZERO,
            trail_timer: Timer::from_seconds(0.03, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct SwordTrail {
    pub timer: Timer,
    pub start_size: Vec2,
}

#[derive(Resource, Debug, Default)]
pub struct SwordAimState {
    pub active: bool,
    pub origin: Vec2,
    pub direction: Vec2,
    pub preview_end: Vec2,
    pub blocked: bool,
}

impl SwordAimState {
    pub fn reset(&mut self) {
        self.active = false;
        self.origin = Vec2::ZERO;
        self.direction = Vec2::ZERO;
        self.preview_end = Vec2::ZERO;
        self.blocked = false;
    }
}

#[derive(Component)]
pub struct SwordAimGuide;

#[derive(Component)]
pub struct SwordAimReticle;

#[derive(Resource)]
pub struct SwordVisualHandles {
    pub spinning_texture: Handle<Image>,
    pub spinning_layout: Handle<TextureAtlasLayout>,
    pub stuck_texture: Handle<Image>,
}
