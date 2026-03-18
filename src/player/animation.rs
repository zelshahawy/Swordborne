use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::player::components::{
    AnimationTimer, CurrentAnimation, Facing, HasSword, IDLE_FPS, JUMP_FPS, OnGround, PLAYER_SCALE,
    Player, PlayerActionState, PlayerAnimState, PlayerAnimationHandles, RUN_FPS, SLASH_FPS,
    SLASH_FRAMES, Velocity,
};

type PlayerAnimQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Velocity,
        &'static OnGround,
        &'static HasSword,
        &'static PlayerActionState,
        &'static mut Sprite,
        &'static mut AnimationTimer,
        &'static mut CurrentAnimation,
    ),
    With<Player>,
>;

pub fn select_animation(anims: Res<PlayerAnimationHandles>, mut query: PlayerAnimQuery) {
    let Ok((velocity, on_ground, has_sword, action_state, mut sprite, mut timer, mut current)) =
        query.single_mut()
    else {
        return;
    };

    let next_state = if *action_state == PlayerActionState::Slash && has_sword.0 {
        PlayerAnimState::Slash
    } else if !on_ground.0 {
        PlayerAnimState::Jump
    } else if velocity.x.abs() > 0.1 {
        PlayerAnimState::Run
    } else {
        PlayerAnimState::Idle
    };

    if next_state == current.state && has_sword.0 == current.with_sword {
        return;
    }

    let (texture, layout, frame_count, fps) = match (has_sword.0, next_state) {
        (true, PlayerAnimState::Idle) => (
            anims.idle_sword_texture.clone(),
            anims.idle_sword_layout.clone(),
            4,
            IDLE_FPS,
        ),
        (false, PlayerAnimState::Idle) => (
            anims.idle_no_sword_texture.clone(),
            anims.idle_no_sword_layout.clone(),
            4,
            IDLE_FPS,
        ),
        (true, PlayerAnimState::Run) => (
            anims.run_sword_texture.clone(),
            anims.run_sword_layout.clone(),
            6,
            RUN_FPS,
        ),
        (false, PlayerAnimState::Run) => (
            anims.run_no_sword_texture.clone(),
            anims.run_no_sword_layout.clone(),
            6,
            RUN_FPS,
        ),
        (true, PlayerAnimState::Jump) => (
            anims.jump_sword_texture.clone(),
            anims.jump_sword_layout.clone(),
            2,
            JUMP_FPS,
        ),
        (false, PlayerAnimState::Jump) => (
            anims.jump_no_sword_texture.clone(),
            anims.jump_no_sword_layout.clone(),
            2,
            JUMP_FPS,
        ),
        (true, PlayerAnimState::Slash) => (
            anims.slash_sword_texture.clone(),
            anims.slash_sword_layout.clone(),
            SLASH_FRAMES,
            SLASH_FPS,
        ),
        (false, PlayerAnimState::Slash) => (
            anims.idle_no_sword_texture.clone(),
            anims.idle_no_sword_layout.clone(),
            4,
            IDLE_FPS,
        ),
    };

    *sprite = Sprite::from_atlas_image(texture, TextureAtlas { layout, index: 0 });

    timer.0 = Timer::from_seconds(1.0 / fps, TimerMode::Repeating);
    current.state = next_state;
    current.frame_count = frame_count;
    current.with_sword = has_sword.0;
}

pub fn animate_player(
    time: Res<Time>,
    mut query: Query<
        (
            &Velocity,
            &PlayerActionState,
            &mut AnimationTimer,
            &CurrentAnimation,
            &mut Sprite,
        ),
        With<Player>,
    >,
) {
    let Ok((velocity, action_state, mut timer, current, mut sprite)) = query.single_mut() else {
        return;
    };

    if current.state == PlayerAnimState::Jump {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = if velocity.y > 0.0 { 1 } else { 0 };
        }
        return;
    }

    if current.state == PlayerAnimState::Slash {
        timer.0.tick(time.delta());

        if let Some(atlas) = &mut sprite.texture_atlas
            && timer.0.just_finished()
        {
            if atlas.index < current.frame_count - 1 {
                atlas.index += 1;
            } else if *action_state != PlayerActionState::Slash {
                atlas.index = 0;
            }
        }

        return;
    }

    timer.0.tick(time.delta());

    if timer.0.just_finished()
        && let Some(atlas) = &mut sprite.texture_atlas
    {
        atlas.index = (atlas.index + 1) % current.frame_count;
    }
}

pub fn update_player_flip(mut query: Query<(&Facing, &mut Transform, &mut Anchor), With<Player>>) {
    let Ok((facing, mut transform, mut anchor)) = query.single_mut() else {
        return;
    };

    transform.scale.x = PLAYER_SCALE * facing.0;
    transform.scale.y = PLAYER_SCALE;

    *anchor = Anchor::BOTTOM_CENTER;
}
