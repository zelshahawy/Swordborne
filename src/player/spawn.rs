use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::player::components::{
    ActionTimer, AnimationTimer, CurrentAnimation, FRAME_SIZE, Facing, HasSword, IDLE_FPS,
    OnGround, PLAYER_SCALE, Player, PlayerActionState, PlayerAnimState, PlayerAnimationHandles,
    SLASH_FRAME_SIZE, SLASH_FRAMES, Velocity,
};

pub fn load_player_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let idle_sword_texture =
        asset_server.load("blue_knight/idle/action/blue_knight_action_idle.png");
    let idle_no_sword_texture = asset_server.load("blue_knight/idle/no_sword/blue_knight_idle.png");

    let run_sword_texture = asset_server.load("blue_knight/run/action/blue_knight_action_run.png");
    let run_no_sword_texture = asset_server.load("blue_knight/run/no_sword/blue_knight_run.png");

    let jump_sword_texture =
        asset_server.load("blue_knight/jump_stop/action/blue_knight_jump_action.png");
    let jump_no_sword_texture =
        asset_server.load("blue_knight/jump_stop/no_sword/blue_knight_jump_strip2.png");

    let slash_sword_texture = asset_server.load("blue_knight/attack/combo1/blue_knight_attack.png");

    let idle_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 4, 1, None, None));
    let idle_no_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 4, 1, None, None));

    let run_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 6, 1, None, None));
    let run_no_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 6, 1, None, None));

    let jump_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 2, 1, None, None));
    let jump_no_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 2, 1, None, None));

    let slash_sword_layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        SLASH_FRAME_SIZE,
        SLASH_FRAMES as u32,
        1,
        None,
        None,
    ));

    commands.insert_resource(PlayerAnimationHandles {
        idle_sword_layout,
        idle_sword_texture,
        idle_no_sword_layout,
        idle_no_sword_texture,
        run_sword_layout,
        run_sword_texture,
        run_no_sword_layout,
        run_no_sword_texture,
        jump_sword_layout,
        jump_sword_texture,
        jump_no_sword_layout,
        jump_no_sword_texture,
        slash_sword_layout,
        slash_sword_texture,
    });
}

pub fn spawn_player_entity(
    commands: &mut Commands,
    anims: &PlayerAnimationHandles,
    position: Vec3,
    has_sword: bool,
) -> Entity {
    let (texture, layout) = if has_sword {
        (
            anims.idle_sword_texture.clone(),
            anims.idle_sword_layout.clone(),
        )
    } else {
        (
            anims.idle_no_sword_texture.clone(),
            anims.idle_no_sword_layout.clone(),
        )
    };

    commands
        .spawn((
            Sprite::from_atlas_image(texture, TextureAtlas { layout, index: 0 }),
            Anchor::BOTTOM_CENTER,
            Transform::from_translation(position).with_scale(Vec3::splat(PLAYER_SCALE)),
            Player,
            Velocity::default(),
            Facing(1.0),
            OnGround(true),
            HasSword(has_sword),
            PlayerActionState::None,
            AnimationTimer(Timer::from_seconds(1.0 / IDLE_FPS, TimerMode::Repeating)),
            ActionTimer(Timer::from_seconds(0.0, TimerMode::Once)),
            CurrentAnimation {
                state: PlayerAnimState::Idle,
                frame_count: 4,
                with_sword: has_sword,
            },
        ))
        .id()
}
