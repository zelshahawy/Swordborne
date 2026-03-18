use bevy::prelude::*;

use crate::level::SwordBlocker;
use crate::player::components::{
    GROUND_Y, OnGround, PLAYER_HALF_WIDTH, PLAYER_HEIGHT, Player, Velocity,
};

type PlayerCollisionQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut Velocity,
        &'static mut OnGround,
    ),
    With<Player>,
>;

type WorldBlockerQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static SwordBlocker), Without<Player>>;

const SUPPORT_SNAP_EPSILON: f32 = 2.0;

pub fn move_player(
    mut player_query: PlayerCollisionQuery,
    blocker_query: WorldBlockerQuery,
    time: Res<Time>,
) {
    let Ok((mut transform, mut velocity, mut on_ground)) = player_query.single_mut() else {
        return;
    };

    let delta_secs = time.delta_secs();
    let mut position = transform.translation;
    let current_bottom = position.y;

    position.x += velocity.x * delta_secs;
    position.x = resolve_horizontal(position.x, current_bottom, velocity.x, &blocker_query);

    if horizontal_collision(position.x, current_bottom, velocity.x, &blocker_query) {
        velocity.x = 0.0;
    }

    position.y += velocity.y * delta_secs;
    on_ground.0 = false;

    if let Some(resolved_y) = resolve_vertical(
        position.x,
        current_bottom,
        position.y,
        velocity.y,
        &blocker_query,
        &mut on_ground,
    ) {
        position.y = resolved_y;
        velocity.y = 0.0;
    } else if let Some(supported_y) = resolve_support(position.x, position.y, &blocker_query) {
        position.y = supported_y;
        velocity.y = 0.0;
        on_ground.0 = true;
    }

    if position.y <= GROUND_Y {
        position.y = GROUND_Y;
        velocity.y = 0.0;
        on_ground.0 = true;
    }

    transform.translation.x = position.x;
    transform.translation.y = position.y;
}

fn resolve_horizontal(
    candidate_x: f32,
    bottom_y: f32,
    velocity_x: f32,
    blocker_query: &WorldBlockerQuery,
) -> f32 {
    let mut resolved_x = candidate_x;

    if velocity_x == 0.0 {
        return resolved_x;
    }

    for (transform, blocker) in blocker_query.iter() {
        let solid_left = transform.translation.x - blocker.half_extents.x;
        let solid_right = transform.translation.x + blocker.half_extents.x;
        let solid_bottom = transform.translation.y - blocker.half_extents.y;
        let solid_top = transform.translation.y + blocker.half_extents.y;

        if player_top(bottom_y) <= solid_bottom || bottom_y >= solid_top {
            continue;
        }

        let player_left = resolved_x - PLAYER_HALF_WIDTH;
        let player_right = resolved_x + PLAYER_HALF_WIDTH;

        if player_right <= solid_left || player_left >= solid_right {
            continue;
        }

        if velocity_x > 0.0 {
            resolved_x = solid_left - PLAYER_HALF_WIDTH;
        } else {
            resolved_x = solid_right + PLAYER_HALF_WIDTH;
        }
    }

    resolved_x
}

fn horizontal_collision(
    candidate_x: f32,
    bottom_y: f32,
    velocity_x: f32,
    blocker_query: &WorldBlockerQuery,
) -> bool {
    if velocity_x == 0.0 {
        return false;
    }

    for (transform, blocker) in blocker_query.iter() {
        let solid_left = transform.translation.x - blocker.half_extents.x;
        let solid_right = transform.translation.x + blocker.half_extents.x;
        let solid_bottom = transform.translation.y - blocker.half_extents.y;
        let solid_top = transform.translation.y + blocker.half_extents.y;

        if player_top(bottom_y) <= solid_bottom || bottom_y >= solid_top {
            continue;
        }

        let player_left = candidate_x - PLAYER_HALF_WIDTH;
        let player_right = candidate_x + PLAYER_HALF_WIDTH;

        if player_right > solid_left && player_left < solid_right {
            return true;
        }
    }

    false
}

fn resolve_vertical(
    center_x: f32,
    previous_bottom_y: f32,
    candidate_bottom_y: f32,
    velocity_y: f32,
    blocker_query: &WorldBlockerQuery,
    on_ground: &mut OnGround,
) -> Option<f32> {
    if velocity_y == 0.0 {
        return None;
    }

    let mut resolved_bottom = candidate_bottom_y;
    let mut collided = false;

    for (transform, blocker) in blocker_query.iter() {
        let solid_left = transform.translation.x - blocker.half_extents.x;
        let solid_right = transform.translation.x + blocker.half_extents.x;
        let solid_bottom = transform.translation.y - blocker.half_extents.y;
        let solid_top = transform.translation.y + blocker.half_extents.y;

        let player_left = center_x - PLAYER_HALF_WIDTH;
        let player_right = center_x + PLAYER_HALF_WIDTH;

        if player_right <= solid_left || player_left >= solid_right {
            continue;
        }

        let candidate_top = player_top(candidate_bottom_y);

        if velocity_y < 0.0 {
            let previous_bottom = previous_bottom_y;
            if previous_bottom >= solid_top && candidate_bottom_y <= solid_top {
                resolved_bottom = resolved_bottom.max(solid_top);
                collided = true;
                on_ground.0 = true;
            }
        } else {
            let previous_top = player_top(previous_bottom_y);
            if previous_top <= solid_bottom && candidate_top >= solid_bottom {
                resolved_bottom = resolved_bottom.min(solid_bottom - PLAYER_HEIGHT);
                collided = true;
            }
        }
    }

    collided.then_some(resolved_bottom)
}

fn resolve_support(center_x: f32, bottom_y: f32, blocker_query: &WorldBlockerQuery) -> Option<f32> {
    let mut supported_y: Option<f32> = None;

    for (transform, blocker) in blocker_query.iter() {
        let solid_left = transform.translation.x - blocker.half_extents.x;
        let solid_right = transform.translation.x + blocker.half_extents.x;
        let solid_top = transform.translation.y + blocker.half_extents.y;

        let player_left = center_x - PLAYER_HALF_WIDTH;
        let player_right = center_x + PLAYER_HALF_WIDTH;

        if player_right <= solid_left || player_left >= solid_right {
            continue;
        }

        if (bottom_y - solid_top).abs() <= SUPPORT_SNAP_EPSILON {
            supported_y = Some(match supported_y {
                Some(current) => current.max(solid_top),
                None => solid_top,
            });
        }
    }

    supported_y
}

fn player_top(bottom_y: f32) -> f32 {
    bottom_y + PLAYER_HEIGHT
}
