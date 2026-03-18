use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::level::LevelBounds;
use crate::player::{Facing, GROUND_Y, HasSword, Player};
use crate::sword::{
    Sword, SwordAnimationTimer, SwordFlight, SwordState, SwordTrail, SwordVelocity,
    SwordVisualHandles,
};

const PICKUP_DISTANCE: f32 = 48.0;
const STUCK_PICKUP_HORIZONTAL_DISTANCE: f32 = 56.0;
const STUCK_PICKUP_VERTICAL_DISTANCE: f32 = 96.0;
const THROW_SPEED_X: f32 = 760.0;
const SWORD_GRAVITY: f32 = -900.0;
const SPINNING_SWORD_FRAMES: usize = 4;
const FLYING_SWORD_HALF_SIZE: f32 = 48.0;
const THROW_VERTICAL_OFFSET: f32 = 10.0 + FLYING_SWORD_HALF_SIZE;
const MAX_STRAIGHT_THROW_DISTANCE: f32 = 820.0;
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

type PlayerThrowQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, &'static Facing, &'static mut HasSword),
    (With<Player>, Without<Sword>),
>;

type SwordThrowQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut SwordState,
        &'static mut SwordVelocity,
        &'static mut Sprite,
        &'static mut SwordFlight,
        &'static mut Anchor,
        &'static mut SwordAnimationTimer,
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

pub fn throw_sword(
    keyboard: Res<ButtonInput<KeyCode>>,
    sword_visuals: Res<SwordVisualHandles>,
    mut player_query: PlayerThrowQuery,
    mut sword_query: SwordThrowQuery,
) {
    if !keyboard.just_pressed(KeyCode::KeyJ) {
        return;
    }

    let Ok((player_transform, facing, mut has_sword)) = player_query.single_mut() else {
        return;
    };

    if !has_sword.0 {
        return;
    }

    for (
        mut sword_transform,
        mut visibility,
        mut sword_state,
        mut sword_velocity,
        mut sprite,
        mut flight,
        mut anchor,
        mut anim_timer,
    ) in &mut sword_query
    {
        if *sword_state != SwordState::Equipped {
            continue;
        }

        has_sword.0 = false;
        *sword_state = SwordState::Flying;
        *visibility = Visibility::Visible;

        sword_transform.translation =
            player_transform.translation + Vec3::new(20.0 * facing.0, THROW_VERTICAL_OFFSET, 0.5);
        sword_transform.rotation = Quat::IDENTITY;

        sword_velocity.x = THROW_SPEED_X * facing.0;
        sword_velocity.y = 0.0;

        *sprite = Sprite::from_atlas_image(
            sword_visuals.spinning_texture.clone(),
            TextureAtlas {
                layout: sword_visuals.spinning_layout.clone(),
                index: 0,
            },
        );
        // Flying should pivot around the sword's center, not the planted hilt.
        *anchor = Anchor::CENTER;
        flight.launch_position = sword_transform.translation.truncate();
        flight.last_trail_position = flight.launch_position;
        flight.trail_timer.reset();
        anim_timer.0.reset();

        break;
    }
}

pub fn update_flying_sword(
    mut commands: Commands,
    time: Res<Time>,
    sword_visuals: Res<SwordVisualHandles>,
    bounds: Res<LevelBounds>,
    mut sword_query: SwordFlightQuery,
) {
    let delta = time.delta();
    let delta_secs = delta.as_secs_f32();
    let ground_contact_y = GROUND_Y + FLYING_SWORD_HALF_SIZE;
    let left_wall_contact_x = bounds.wall_left_x + FLYING_SWORD_HALF_SIZE;
    let right_wall_contact_x = bounds.wall_right_x - FLYING_SWORD_HALF_SIZE;

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

        if transform.translation.x <= left_wall_contact_x {
            transform.translation.x = bounds.wall_left_x;
            velocity.x = 0.0;
            velocity.y = 0.0;
            transform.rotation = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);
            *sword_state = SwordState::Stuck;
            set_resting_sword_visual(&mut sprite, &mut anchor, &sword_visuals);
            reset_flight_trail(&mut flight, &transform);
            continue;
        }

        if transform.translation.x >= right_wall_contact_x {
            transform.translation.x = bounds.wall_right_x;
            velocity.x = 0.0;
            velocity.y = 0.0;
            transform.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
            *sword_state = SwordState::Stuck;
            set_resting_sword_visual(&mut sprite, &mut anchor, &sword_visuals);
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
