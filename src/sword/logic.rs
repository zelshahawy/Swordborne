use bevy::prelude::*;

use crate::player::{Facing, GROUND_Y, HasSword, Player};
use crate::sword::{Sword, SwordState, SwordVelocity, SwordVisualHandles};

const PICKUP_DISTANCE: f32 = 40.0;
const THROW_SPEED_X: f32 = 420.0;
const THROW_SPEED_Y: f32 = 360.0;
const SWORD_GRAVITY: f32 = -900.0;
const LEFT_WALL_X: f32 = -600.0;
const RIGHT_WALL_X: f32 = 600.0;

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
    ),
    With<Sword>,
>;

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

        let distance = player_transform
            .translation
            .distance(sword_transform.translation);

        if distance <= PICKUP_DISTANCE {
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

    for (mut sword_transform, mut visibility, mut sword_state, mut sword_velocity, mut sprite) in
        &mut sword_query
    {
        if *sword_state != SwordState::Equipped {
            continue;
        }

        has_sword.0 = false;
        *sword_state = SwordState::Flying;
        *visibility = Visibility::Visible;

        sword_transform.translation =
            player_transform.translation + Vec3::new(20.0 * facing.0, 10.0, 0.5);
        sword_transform.rotation = Quat::IDENTITY;

        sword_velocity.x = THROW_SPEED_X * facing.0;
        sword_velocity.y = THROW_SPEED_Y;

        *sprite = Sprite::from_image(sword_visuals.spinning_texture.clone());

        break;
    }
}

pub fn update_flying_sword(
    time: Res<Time>,
    sword_visuals: Res<SwordVisualHandles>,
    mut sword_query: SwordFlightQuery,
) {
    for (mut transform, mut sword_state, mut velocity, mut sprite) in &mut sword_query {
        if *sword_state != SwordState::Flying {
            continue;
        }

        velocity.y += SWORD_GRAVITY * time.delta_secs();
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();

        transform.rotate_z(12.0 * time.delta_secs());

        if transform.translation.y <= GROUND_Y {
            transform.translation.y = GROUND_Y;
            velocity.x = 0.0;
            velocity.y = 0.0;
            transform.rotation = Quat::IDENTITY;
            *sword_state = SwordState::Stuck;
            *sprite = Sprite::from_image(sword_visuals.stuck_texture.clone());
            continue;
        }

        if transform.translation.x <= LEFT_WALL_X {
            transform.translation.x = LEFT_WALL_X;
            velocity.x = 0.0;
            velocity.y = 0.0;
            transform.rotation = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);
            *sword_state = SwordState::Stuck;
            *sprite = Sprite::from_image(sword_visuals.stuck_texture.clone());
            continue;
        }

        if transform.translation.x >= RIGHT_WALL_X {
            transform.translation.x = RIGHT_WALL_X;
            velocity.x = 0.0;
            velocity.y = 0.0;
            transform.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
            *sword_state = SwordState::Stuck;
            *sprite = Sprite::from_image(sword_visuals.stuck_texture.clone());
        }
    }
}
