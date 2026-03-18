use bevy::prelude::*;

use crate::player::components::{
    AnimationTimer, CurrentAnimation, Facing, HasSword, IDLE_FPS, JUMP_FPS, PLAYER_SCALE,
    Player, PlayerAnimState, PlayerAnimationHandles, RUN_FPS, Velocity, OnGround,
};

pub fn select_animation(
    anims: Res<PlayerAnimationHandles>,
    mut query: Query<
        (
            &Velocity,
            &OnGround,
            &HasSword,
            &mut Sprite,
            &mut AnimationTimer,
            &mut CurrentAnimation,
        ),
        With<Player>,
    >,
) {
    let Ok((velocity, on_ground, has_sword, mut sprite, mut timer, mut current)) =
        query.single_mut()
    else {
        return;
    };

    let next_state = if !on_ground.0 {
        PlayerAnimState::Jump
    } else if velocity.x.abs() > 0.1 {
        PlayerAnimState::Run
    } else {
        PlayerAnimState::Idle
    };

    if next_state == current.state {
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
    };

    *sprite = Sprite::from_atlas_image(
        texture,
        TextureAtlas {
            layout,
            index: 0,
        },
    );

    timer.0 = Timer::from_seconds(1.0 / fps, TimerMode::Repeating);
    current.state = next_state;
    current.frame_count = frame_count;
}

pub fn animate_player(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut AnimationTimer, &CurrentAnimation, &mut Sprite), With<Player>>,
) {
    let Ok((velocity, mut timer, current, mut sprite)) = query.single_mut() else {
        return;
    };

    if current.state == PlayerAnimState::Jump {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = if velocity.y > 0.0 { 1 } else { 0 };
        }
        return;
    }

    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = (atlas.index + 1) % current.frame_count;
        }
    }
}

pub fn update_player_flip(mut query: Query<(&Facing, &mut Transform), With<Player>>) {
    let Ok((facing, mut transform)) = query.single_mut() else {
        return;
    };

    transform.scale.x = PLAYER_SCALE * facing.0;
    transform.scale.y = PLAYER_SCALE;
}
