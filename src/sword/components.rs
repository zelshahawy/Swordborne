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

#[derive(Resource)]
pub struct SwordVisualHandles {
    pub spinning_texture: Handle<Image>,
    pub spinning_layout: Handle<TextureAtlasLayout>,
    pub stuck_texture: Handle<Image>,
}
