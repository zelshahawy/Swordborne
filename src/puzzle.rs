use bevy::prelude::*;

use crate::sword::{Sword, SwordState};

pub struct PuzzlePlugin;

const TARGET_RADIUS: f32 = 36.0;

const TARGET_INACTIVE_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);
const TARGET_ACTIVE_COLOR: Color = Color::srgb(0.2, 0.9, 0.3);

const DOOR_CLOSED_COLOR: Color = Color::srgb(0.3, 0.3, 0.8);
const DOOR_OPEN_COLOR: Color = Color::srgb(0.5, 0.9, 1.0);

#[derive(Component)]
pub struct SwordTarget {
    pub active: bool,
}

#[derive(Component)]
pub struct Door {
    pub open: bool,
}

#[derive(Component)]
pub struct LinkedPuzzle {
    pub id: u32,
}

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_test_puzzle)
            .add_systems(Update, (update_targets_from_sword, update_doors_visuals));
    }
}

fn spawn_test_puzzle(mut commands: Commands) {
    let puzzle_id = 1;

    commands.spawn((
        Sprite::from_color(TARGET_INACTIVE_COLOR, Vec2::new(32.0, 32.0)),
        Transform::from_xyz(250.0, -170.0, 1.0),
        SwordTarget { active: false },
        LinkedPuzzle { id: puzzle_id },
    ));

    commands.spawn((
        Sprite::from_color(DOOR_CLOSED_COLOR, Vec2::new(40.0, 140.0)),
        Transform::from_xyz(420.0, -180.0, 1.0),
        Door { open: false },
        LinkedPuzzle { id: puzzle_id },
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.15, 0.15, 0.15), Vec2::new(40.0, 160.0)),
        Transform::from_xyz(-600.0, -170.0, 0.0),
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.15, 0.15, 0.15), Vec2::new(40.0, 160.0)),
        Transform::from_xyz(600.0, -170.0, 0.0),
    ));
}

fn update_targets_from_sword(
    sword_query: Query<(&Transform, &SwordState), With<Sword>>,
    mut target_query: Query<(&Transform, &mut SwordTarget, &LinkedPuzzle, &mut Sprite)>,
    mut door_query: Query<(&LinkedPuzzle, &mut Door)>,
) {
    let mut active_puzzle_ids = Vec::new();

    for (_, mut target, _, mut sprite) in &mut target_query {
        target.active = false;
        sprite.color = TARGET_INACTIVE_COLOR;
    }

    for (sword_transform, sword_state) in &sword_query {
        if *sword_state != SwordState::Stuck {
            continue;
        }

        for (target_transform, mut target, linked_puzzle, mut sprite) in &mut target_query {
            let distance = sword_transform
                .translation
                .distance(target_transform.translation);

            if distance <= TARGET_RADIUS {
                target.active = true;
                sprite.color = TARGET_ACTIVE_COLOR;
                active_puzzle_ids.push(linked_puzzle.id);
            }
        }
    }

    for (door_link, mut door) in &mut door_query {
        door.open = active_puzzle_ids.contains(&door_link.id);
    }
}

fn update_doors_visuals(mut door_query: Query<(&Door, &mut Sprite, &mut Transform)>) {
    for (door, mut sprite, mut transform) in &mut door_query {
        if door.open {
            sprite.color = DOOR_OPEN_COLOR;
            transform.scale.y = 0.2;
        } else {
            sprite.color = DOOR_CLOSED_COLOR;
            transform.scale.y = 1.0;
        }
    }
}
