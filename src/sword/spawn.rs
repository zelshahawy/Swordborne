use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::sword::{
    Sword, SwordAnimationTimer, SwordFlight, SwordState, SwordVelocity, SwordVisualHandles,
};

const SWORD_SCALE: f32 = 4.0;
const SPINNING_SWORD_FPS: f32 = 12.0;
const SPINNING_SWORD_FRAME_SIZE: UVec2 = UVec2::new(24, 24);

pub fn load_sword_visuals(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let spinning_texture = asset_server.load("blue_knight/sword/spinning_sword.png");
    let stuck_texture = asset_server.load("blue_knight/sword/stuck_sword.png");

    let spinning_layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        SPINNING_SWORD_FRAME_SIZE,
        4,
        1,
        None,
        None,
    ));

    commands.insert_resource(SwordVisualHandles {
        spinning_texture,
        spinning_layout,
        stuck_texture,
    });
}

pub fn spawn_sword_entity(
    commands: &mut Commands,
    sword_visuals: &SwordVisualHandles,
    position: Vec3,
    state: SwordState,
) -> Entity {
    let visibility = if state == SwordState::Equipped {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };

    commands
        .spawn((
            Sprite::from_image(sword_visuals.stuck_texture.clone()),
            Anchor::BOTTOM_CENTER,
            Transform::from_translation(position).with_scale(Vec3::splat(SWORD_SCALE)),
            visibility,
            Sword,
            state,
            SwordVelocity::default(),
            SwordFlight::default(),
            SwordAnimationTimer(Timer::from_seconds(
                1.0 / SPINNING_SWORD_FPS,
                TimerMode::Repeating,
            )),
        ))
        .id()
}
