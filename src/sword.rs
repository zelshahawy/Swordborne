use bevy::prelude::*;

use crate::player::{Facing, GROUND_Y, HasSword, Player};

pub struct SwordPlugin;

const THROW_SPEED_X: f32 = 420.0;
const THROW_SPEED_Y: f32 = 360.0;
const SWORD_GRAVITY: f32 = -900.0;

const LEFT_WALL_X: f32 = -600.0;
const RIGHT_WALL_X: f32 = 600.0;

const RECALL_DISTANCE: f32 = 60.0;

#[derive(Component)]
pub struct Sword;

#[derive(Component, Debug)]
pub struct SwordVelocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug)]
pub struct Stuck;

#[derive(Resource, Default)]
pub struct SwordState {
    pub active: bool,
}

impl Plugin for SwordPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SwordState>()
            .add_systems(Update, (throw_sword, update_sword_motion, recall_sword));
    }
}

fn throw_sword(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sword_state: ResMut<SwordState>,
    mut player_query: Query<(&Transform, &Facing, &mut HasSword), With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyJ) || sword_state.active {
        return;
    }

    let Ok((player_transform, facing, mut has_sword)) = player_query.single_mut() else {
        return;
    };

    if !has_sword.0 {
        return;
    }

    let start = player_transform.translation + Vec3::new(20.0 * facing.0, 10.0, 0.0);

    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.9, 0.2), Vec2::new(24.0, 8.0)),
        Transform::from_translation(start),
        Sword,
        SwordVelocity {
            x: THROW_SPEED_X * facing.0,
            y: THROW_SPEED_Y,
        },
    ));

    has_sword.0 = false;
    sword_state.active = true;
}

fn update_sword_motion(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut SwordVelocity, Option<&Stuck>), With<Sword>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut velocity, stuck) in &mut query {
        if stuck.is_some() {
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
            transform.rotation = Quat::from_rotation_z(0.0);
            commands.entity(entity).insert(Stuck);
            continue;
        }

        if transform.translation.x <= LEFT_WALL_X {
            transform.translation.x = LEFT_WALL_X;
            velocity.x = 0.0;
            velocity.y = 0.0;
            transform.rotation = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);
            commands.entity(entity).insert(Stuck);
            continue;
        }

        if transform.translation.x >= RIGHT_WALL_X {
            transform.translation.x = RIGHT_WALL_X;
            velocity.x = 0.0;
            velocity.y = 0.0;
            transform.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
            commands.entity(entity).insert(Stuck);
        }
    }
}

fn recall_sword(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sword_state: ResMut<SwordState>,
    sword_query: Query<(Entity, &Transform), With<Sword>>,
    player_query: Query<&Transform, With<Player>>,
    mut player_has_sword_query: Query<&mut HasSword, With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyK) {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok(mut has_sword) = player_has_sword_query.single_mut() else {
        return;
    };

    let mut recalled_any = false;

    for (entity, sword_transform) in &sword_query {
        let distance = player_transform
            .translation
            .distance(sword_transform.translation);

        if distance <= RECALL_DISTANCE {
            commands.entity(entity).despawn();
            recalled_any = true;
        }
    }

    if recalled_any {
        has_sword.0 = true;
        sword_state.active = false;
    }
}
