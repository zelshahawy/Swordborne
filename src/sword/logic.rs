use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::level::{LevelBounds, SwordBlocker};
use crate::player::{GROUND_Y, HasSword, Player};
use crate::sword::{
    Sword, SwordAnimationTimer, SwordFlight, SwordState, SwordTrail, SwordVelocity,
    SwordVisualHandles,
};

const PICKUP_DISTANCE: f32 = 48.0;
const STUCK_PICKUP_HORIZONTAL_DISTANCE: f32 = 56.0;
const STUCK_PICKUP_VERTICAL_DISTANCE: f32 = 96.0;
pub(crate) const THROW_SPEED: f32 = 920.0;
const SWORD_GRAVITY: f32 = -900.0;
const SPINNING_SWORD_FRAMES: usize = 4;
pub(crate) const FLYING_SWORD_HALF_SIZE: f32 = 48.0;
const THROW_HAND_HORIZONTAL_OFFSET: f32 = 24.0;
const THROW_HAND_VERTICAL_OFFSET: f32 = 58.0;
pub(crate) const MAX_STRAIGHT_THROW_DISTANCE: f32 = 980.0;
const DROP_HORIZONTAL_DRAG: f32 = 1400.0;
const DROP_SPIN_RATE: f32 = 10.0;
const TRAIL_LIFETIME: f32 = 0.18;
const TRAIL_MIN_SEGMENT_LENGTH: f32 = 18.0;
const TRAIL_EXTRA_LENGTH: f32 = 28.0;
const TRAIL_WIDTH: f32 = 14.0;
const TRAIL_ALPHA: f32 = 0.65;
const TRAIL_RED: f32 = 0.95;
const TRAIL_GREEN: f32 = 0.98;
const TRAIL_BLUE: f32 = 1.0;
const EPSILON: f32 = 0.001;

type PlayerPickupQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static mut HasSword), (With<Player>, Without<Sword>)>;

type SwordPickupQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        &'static mut Visibility,
        &'static mut SwordState,
    ),
    (With<Sword>, Without<Player>),
>;

type SwordFlightQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut SwordState,
        &'static mut SwordVelocity,
        &'static mut Sprite,
        &'static mut SwordFlight,
        &'static mut Anchor,
        &'static mut SwordAnimationTimer,
    ),
    With<Sword>,
>;

type SwordTrailQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static mut Sprite, &'static mut SwordTrail)>;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RayHit {
    pub distance: f32,
    pub point: Vec2,
    pub normal: Vec2,
}

pub fn pickup_sword(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: PlayerPickupQuery,
    mut sword_query: SwordPickupQuery,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok((player_transform, mut has_sword)) = player_query.single_mut() else {
        return;
    };

    if has_sword.0 {
        return;
    }

    for (sword_transform, mut visibility, mut sword_state) in &mut sword_query {
        if *sword_state != SwordState::Grounded && *sword_state != SwordState::Stuck {
            continue;
        }

        if can_pick_up_sword(player_transform, sword_transform, *sword_state) {
            has_sword.0 = true;
            *sword_state = SwordState::Equipped;
            *visibility = Visibility::Hidden;
            break;
        }
    }
}

pub fn update_flying_sword(
    mut commands: Commands,
    time: Res<Time>,
    sword_visuals: Res<SwordVisualHandles>,
    bounds: Res<LevelBounds>,
    blocker_query: Query<(&Transform, &SwordBlocker), Without<Sword>>,
    mut sword_query: SwordFlightQuery,
) {
    let delta = time.delta();
    let delta_secs = delta.as_secs_f32();
    let ground_contact_y = GROUND_Y + FLYING_SWORD_HALF_SIZE;

    for (
        mut transform,
        mut sword_state,
        mut velocity,
        mut sprite,
        mut flight,
        mut anchor,
        mut anim_timer,
    ) in &mut sword_query
    {
        match *sword_state {
            SwordState::Flying | SwordState::Dropping => {}
            _ => {
                reset_flight_trail(&mut flight, &transform);
                continue;
            }
        }

        anim_timer.0.tick(delta);
        if anim_timer.0.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            atlas.index = (atlas.index + 1) % SPINNING_SWORD_FRAMES;
        }

        let start = transform.translation.truncate();

        match *sword_state {
            SwordState::Flying => {
                transform.translation.x += velocity.x * delta_secs;
                transform.translation.y += velocity.y * delta_secs;
                transform.rotation = Quat::IDENTITY;

                maybe_emit_sword_trail(&mut commands, delta, &mut flight, &transform);

                let traveled = transform
                    .translation
                    .truncate()
                    .distance(flight.launch_position);
                if traveled >= MAX_STRAIGHT_THROW_DISTANCE {
                    *sword_state = SwordState::Dropping;
                    velocity.y = 0.0;
                    reset_flight_trail(&mut flight, &transform);
                }
            }
            SwordState::Dropping => {
                velocity.y += SWORD_GRAVITY * delta_secs;
                velocity.x = move_toward_zero(velocity.x, DROP_HORIZONTAL_DRAG * delta_secs);
                transform.translation.x += velocity.x * delta_secs;
                transform.translation.y += velocity.y * delta_secs;

                let spin_direction = if velocity.x < 0.0 { -1.0 } else { 1.0 };
                transform.rotate_z(spin_direction * DROP_SPIN_RATE * delta_secs);
                reset_flight_trail(&mut flight, &transform);
            }
            _ => unreachable!(),
        }

        let end = transform.translation.truncate();
        if let Some(hit) = sweep_against_blockers(start, end, &blocker_query) {
            transform.translation = hit.point.extend(transform.translation.z);
            velocity.x = 0.0;
            velocity.y = 0.0;
            transform.rotation = stuck_rotation_from_normal(hit.normal);
            *sword_state = SwordState::Stuck;
            set_resting_sword_visual(&mut sprite, &mut anchor, &sword_visuals);
            reset_flight_trail(&mut flight, &transform);
            continue;
        }

        if transform.translation.x <= bounds.wall_left_x {
            transform.translation.x = bounds.wall_left_x;
            velocity.x = 0.0;
            velocity.y = 0.0;
            transform.rotation = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);
            *sword_state = SwordState::Stuck;
            set_resting_sword_visual(&mut sprite, &mut anchor, &sword_visuals);
            reset_flight_trail(&mut flight, &transform);
            continue;
        }

        if transform.translation.x >= bounds.wall_right_x {
            transform.translation.x = bounds.wall_right_x;
            velocity.x = 0.0;
            velocity.y = 0.0;
            transform.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
            *sword_state = SwordState::Stuck;
            set_resting_sword_visual(&mut sprite, &mut anchor, &sword_visuals);
            reset_flight_trail(&mut flight, &transform);
            continue;
        }

        if *sword_state == SwordState::Flying && transform.translation.y >= bounds.ceiling_y {
            transform.translation.y = bounds.ceiling_y;
            velocity.y = 0.0;
            *sword_state = SwordState::Dropping;
            reset_flight_trail(&mut flight, &transform);
            continue;
        }

        if transform.translation.y <= ground_contact_y {
            transform.translation.y = GROUND_Y;
            velocity.x = 0.0;
            velocity.y = 0.0;
            transform.rotation = Quat::IDENTITY;
            *sword_state = if *sword_state == SwordState::Dropping {
                SwordState::Grounded
            } else {
                SwordState::Stuck
            };
            set_resting_sword_visual(&mut sprite, &mut anchor, &sword_visuals);
            reset_flight_trail(&mut flight, &transform);
        }
    }
}

pub fn update_sword_trail(
    mut commands: Commands,
    time: Res<Time>,
    mut trail_query: SwordTrailQuery,
) {
    for (entity, mut sprite, mut trail) in &mut trail_query {
        trail.timer.tick(time.delta());

        let remaining = 1.0
            - (trail.timer.elapsed_secs() / trail.timer.duration().as_secs_f32()).clamp(0.0, 1.0);
        let alpha = TRAIL_ALPHA * remaining * remaining;

        sprite.color = Color::srgba(TRAIL_RED, TRAIL_GREEN, TRAIL_BLUE, alpha);
        sprite.custom_size = Some(Vec2::new(
            trail.start_size.x * (0.75 + 0.25 * remaining),
            trail.start_size.y * (0.3 + 0.7 * remaining),
        ));

        if trail.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn sword_launch_origin(player_transform: &Transform, direction: Vec2) -> Vec3 {
    let horizontal_sign = if direction.x.abs() > 0.15 {
        direction.x.signum()
    } else {
        0.0
    };

    player_transform.translation
        + Vec3::new(
            horizontal_sign * THROW_HAND_HORIZONTAL_OFFSET,
            THROW_HAND_VERTICAL_OFFSET,
            0.5,
        )
}

pub(crate) fn raycast_against_bounds(
    origin: Vec2,
    direction: Vec2,
    max_distance: f32,
    bounds: &LevelBounds,
) -> Option<RayHit> {
    if direction.length_squared() <= EPSILON {
        return None;
    }

    let ground_contact_y = GROUND_Y + FLYING_SWORD_HALF_SIZE;
    let mut best_hit: Option<RayHit> = None;

    maybe_take_closer_hit(
        &mut best_hit,
        plane_hit(
            origin,
            direction,
            max_distance,
            bounds.wall_left_x,
            true,
            Vec2::new(1.0, 0.0),
            ground_contact_y,
            bounds.ceiling_y,
        ),
    );
    maybe_take_closer_hit(
        &mut best_hit,
        plane_hit(
            origin,
            direction,
            max_distance,
            bounds.wall_right_x,
            true,
            Vec2::new(-1.0, 0.0),
            ground_contact_y,
            bounds.ceiling_y,
        ),
    );
    maybe_take_closer_hit(
        &mut best_hit,
        plane_hit(
            origin,
            direction,
            max_distance,
            ground_contact_y,
            false,
            Vec2::new(0.0, 1.0),
            bounds.wall_left_x,
            bounds.wall_right_x,
        ),
    );
    best_hit
}

pub(crate) fn raycast_against_aabb(
    origin: Vec2,
    direction: Vec2,
    max_distance: f32,
    center: Vec2,
    half_extents: Vec2,
) -> Option<RayHit> {
    if direction.length_squared() <= EPSILON {
        return None;
    }

    let min = center - half_extents;
    let max = center + half_extents;
    let mut t_min = 0.0;
    let mut t_max = max_distance;
    let mut hit_normal = Vec2::ZERO;

    for axis in 0..2 {
        let (origin_axis, direction_axis, min_axis, max_axis, axis_normal) = if axis == 0 {
            (origin.x, direction.x, min.x, max.x, Vec2::X)
        } else {
            (origin.y, direction.y, min.y, max.y, Vec2::Y)
        };

        if direction_axis.abs() <= EPSILON {
            if origin_axis < min_axis || origin_axis > max_axis {
                return None;
            }
            continue;
        }

        let inverse = 1.0 / direction_axis;
        let mut t1 = (min_axis - origin_axis) * inverse;
        let mut t2 = (max_axis - origin_axis) * inverse;
        let mut normal1 = -axis_normal;
        let mut normal2 = axis_normal;

        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
            std::mem::swap(&mut normal1, &mut normal2);
        }

        if t1 > t_min {
            t_min = t1;
            hit_normal = normal1;
        }

        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }

    if t_min <= EPSILON || t_min > max_distance {
        return None;
    }

    Some(RayHit {
        distance: t_min,
        point: origin + direction * t_min,
        normal: hit_normal,
    })
}

fn sweep_against_blockers(
    start: Vec2,
    end: Vec2,
    blocker_query: &Query<(&Transform, &SwordBlocker), Without<Sword>>,
) -> Option<RayHit> {
    let delta = end - start;
    let distance = delta.length();
    if distance <= EPSILON {
        return None;
    }

    let direction = delta / distance;
    let mut best_hit: Option<RayHit> = None;

    for (transform, blocker) in blocker_query.iter() {
        let expanded_extents = blocker.half_extents + Vec2::splat(FLYING_SWORD_HALF_SIZE - 2.0);
        maybe_take_closer_hit(
            &mut best_hit,
            raycast_against_aabb(
                start,
                direction,
                distance,
                transform.translation.truncate(),
                expanded_extents,
            ),
        );
    }

    best_hit
}

fn maybe_emit_sword_trail(
    commands: &mut Commands,
    delta: std::time::Duration,
    flight: &mut SwordFlight,
    transform: &Transform,
) {
    flight.trail_timer.tick(delta);
    if !flight.trail_timer.just_finished() {
        return;
    }

    let current_position = transform.translation.truncate();
    let segment = current_position - flight.last_trail_position;
    let segment_length = segment.length();

    if segment_length < 1.0 {
        return;
    }

    let angle = segment.y.atan2(segment.x);
    let midpoint = (flight.last_trail_position + current_position) * 0.5;
    let size = Vec2::new(
        (segment_length + TRAIL_EXTRA_LENGTH).max(TRAIL_MIN_SEGMENT_LENGTH),
        TRAIL_WIDTH,
    );

    commands.spawn((
        Sprite {
            color: Color::srgba(TRAIL_RED, TRAIL_GREEN, TRAIL_BLUE, TRAIL_ALPHA),
            custom_size: Some(size),
            ..Default::default()
        },
        Transform::from_xyz(midpoint.x, midpoint.y, transform.translation.z - 0.1)
            .with_rotation(Quat::from_rotation_z(angle)),
        Visibility::Visible,
        SwordTrail {
            timer: Timer::from_seconds(TRAIL_LIFETIME, TimerMode::Once),
            start_size: size,
        },
    ));

    flight.last_trail_position = current_position;
}

fn move_toward_zero(value: f32, amount: f32) -> f32 {
    if value > 0.0 {
        (value - amount).max(0.0)
    } else {
        (value + amount).min(0.0)
    }
}

fn reset_flight_trail(flight: &mut SwordFlight, transform: &Transform) {
    flight.last_trail_position = transform.translation.truncate();
    flight.trail_timer.reset();
}

fn stuck_rotation_from_normal(normal: Vec2) -> Quat {
    if normal.x < -0.5 {
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
    } else if normal.x > 0.5 {
        Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)
    } else if normal.y < -0.5 {
        Quat::from_rotation_z(std::f32::consts::PI)
    } else {
        Quat::IDENTITY
    }
}

fn set_resting_sword_visual(
    sprite: &mut Sprite,
    anchor: &mut Anchor,
    sword_visuals: &SwordVisualHandles,
) {
    *sprite = Sprite::from_image(sword_visuals.stuck_texture.clone());
    *anchor = Anchor::BOTTOM_CENTER;
}

fn can_pick_up_sword(
    player_transform: &Transform,
    sword_transform: &Transform,
    sword_state: SwordState,
) -> bool {
    let offset = sword_transform.translation - player_transform.translation;

    if offset.length() <= PICKUP_DISTANCE {
        return true;
    }

    sword_state == SwordState::Stuck
        && offset.x.abs() <= STUCK_PICKUP_HORIZONTAL_DISTANCE
        && offset.y.abs() <= STUCK_PICKUP_VERTICAL_DISTANCE
}

fn maybe_take_closer_hit(best_hit: &mut Option<RayHit>, candidate: Option<RayHit>) {
    let Some(candidate) = candidate else {
        return;
    };

    if best_hit.is_none_or(|current| candidate.distance < current.distance) {
        *best_hit = Some(candidate);
    }
}

fn plane_hit(
    origin: Vec2,
    direction: Vec2,
    max_distance: f32,
    plane_value: f32,
    vertical_plane: bool,
    normal: Vec2,
    min_other_axis: f32,
    max_other_axis: f32,
) -> Option<RayHit> {
    let axis_origin = if vertical_plane { origin.x } else { origin.y };
    let axis_direction = if vertical_plane {
        direction.x
    } else {
        direction.y
    };

    if axis_direction.abs() <= EPSILON {
        return None;
    }

    let distance = (plane_value - axis_origin) / axis_direction;
    if distance <= EPSILON || distance > max_distance {
        return None;
    }

    let point = origin + direction * distance;
    let other_axis = if vertical_plane { point.y } else { point.x };
    if other_axis < min_other_axis || other_axis > max_other_axis {
        return None;
    }

    Some(RayHit {
        distance,
        point,
        normal,
    })
}
