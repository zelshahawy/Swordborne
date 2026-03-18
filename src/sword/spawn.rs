use bevy::prelude::*;

use crate::player::GROUND_Y;
use crate::sword::{Sword, SwordState, SwordVelocity, SwordVisualHandles};

const SWORD_SCALE: f32 = 4.0;

pub fn load_sword_visuals(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SwordVisualHandles {
        spinning_texture: asset_server.load("blue_knight/sword/spinning_sword.png"),
        stuck_texture: asset_server.load("blue_knight/sword/stuck_sword.png"),
    });
}

pub fn spawn_sword_at_start(mut commands: Commands, sword_visuals: Res<SwordVisualHandles>) {
    commands.spawn((
        Sprite::from_image(sword_visuals.stuck_texture.clone()),
        Transform::from_xyz(-300.0, GROUND_Y, 1.0).with_scale(Vec3::splat(SWORD_SCALE)),
        Visibility::Visible,
        Sword,
        SwordState::Grounded,
        SwordVelocity::default(),
    ));
}
