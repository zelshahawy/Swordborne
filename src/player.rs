use bevy::prelude::*;

pub struct PlayerPlugin;

pub const GROUND_Y: f32 = -250.0;

const PLAYER_SPEED: f32 = 300.0;
const JUMP_VELOCITY: f32 = 500.0;
const GRAVITY: f32 = -1200.0;

const PLAYER_WITH_SWORD_COLOR: Color = Color::srgb(0.3, 0.7, 0.9);
const PLAYER_SWORDLESS_COLOR: Color = Color::srgb(0.6, 0.4, 0.7);

#[derive(Component)]
pub struct Player;

#[derive(Component, Default, Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug)]
pub struct Facing(pub f32);

#[derive(Component, Debug)]
pub struct OnGround(pub bool);

#[derive(Component, Debug)]
pub struct HasSword(pub bool);

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (
                player_input,
                apply_gravity,
                move_player,
                update_player_visuals,
            )
                .chain(),
        );
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(PLAYER_WITH_SWORD_COLOR, Vec2::new(32.0, 48.0)),
        Transform::from_xyz(0.0, GROUND_Y, 1.0),
        Player,
        Velocity::default(),
        Facing(1.0),
        OnGround(true),
        HasSword(true),
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::new(1200.0, 40.0)),
        Transform::from_xyz(0.0, GROUND_Y - 40.0, 0.0),
    ));
}

fn player_input(
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

fn apply_gravity(mut query: Query<(&mut Velocity, &OnGround), With<Player>>, time: Res<Time>) {
    let Ok((mut velocity, on_ground)) = query.single_mut() else {
        return;
    };

    if !on_ground.0 {
        velocity.y += GRAVITY * time.delta_secs();
    }
}

fn move_player(
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

fn update_player_visuals(mut query: Query<(&HasSword, &mut Sprite), With<Player>>) {
    let Ok((has_sword, mut sprite)) = query.single_mut() else {
        return;
    };

    sprite.color = if has_sword.0 {
        PLAYER_WITH_SWORD_COLOR
    } else {
        PLAYER_SWORDLESS_COLOR
    };
}
