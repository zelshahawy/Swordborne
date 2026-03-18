use bevy::prelude::*;

use crate::player::components::{
    Facing, GRAVITY, JUMP_VELOCITY, OnGround, PLAYER_SPEED, Player, Velocity,
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
