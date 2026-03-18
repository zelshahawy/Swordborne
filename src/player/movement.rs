use bevy::prelude::*;

use crate::player::components::{
    Facing, GRAVITY, GROUND_Y, JUMP_VELOCITY, OnGround, PLAYER_SPEED, Player, Velocity,
};

pub fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Facing, &OnGround), With<Player>>,
) {
    let Ok((mut velocity, mut facing, on_ground)) = query.single_mut() else {
        return;
    };

    let mut direction = 0.0;

    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    velocity.x = direction * PLAYER_SPEED;

    if direction != 0.0 {
        facing.0 = direction.signum();
    }

    if (keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::KeyW))
        && on_ground.0
    {
        velocity.y = JUMP_VELOCITY;
    }
}

pub fn apply_gravity(mut query: Query<(&mut Velocity, &OnGround), With<Player>>, time: Res<Time>) {
    let Ok((mut velocity, on_ground)) = query.single_mut() else {
        return;
    };

    if !on_ground.0 {
        velocity.y += GRAVITY * time.delta_secs();
    }
}

pub fn move_player(
    mut query: Query<(&mut Transform, &mut Velocity, &mut OnGround), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut velocity, mut on_ground)) = query.single_mut() else {
        return;
    };

    transform.translation.x += velocity.x * time.delta_secs();
    transform.translation.y += velocity.y * time.delta_secs();

    if transform.translation.y <= GROUND_Y {
        transform.translation.y = GROUND_Y;
        velocity.y = 0.0;
        on_ground.0 = true;
    } else {
        on_ground.0 = false;
    }
}
