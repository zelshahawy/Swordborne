use bevy::prelude::*;

#[derive(Component)]
pub struct Sword;

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum SwordState {
    Grounded,
    Equipped,
    Flying,
    Stuck,
}

#[derive(Component, Debug, Default)]
pub struct SwordVelocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Resource)]
pub struct SwordVisualHandles {
    pub spinning_texture: Handle<Image>,
    pub stuck_texture: Handle<Image>,
}
