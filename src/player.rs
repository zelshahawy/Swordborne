use bevy::prelude::*;

pub struct PlayerPlugin;

pub const GROUND_Y: f32 = -250.0;

const PLAYER_SPEED: f32 = 300.0;
const JUMP_VELOCITY: f32 = 500.0;
const GRAVITY: f32 = -1200.0;

const FRAME_SIZE: UVec2 = UVec2::new(24, 24);
const PLAYER_SCALE: f32 = 4.0;

const IDLE_FPS: f32 = 6.0;
const RUN_FPS: f32 = 10.0;
const JUMP_FPS: f32 = 8.0;

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

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerAnimState {
    Idle,
    Run,
    Jump,
}

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub struct CurrentAnimation {
    pub state: PlayerAnimState,
    pub frame_count: usize,
}

#[derive(Resource)]
pub struct PlayerAnimationHandles {
    pub idle_sword_layout: Handle<TextureAtlasLayout>,
    pub idle_sword_texture: Handle<Image>,

    pub idle_no_sword_layout: Handle<TextureAtlasLayout>,
    pub idle_no_sword_texture: Handle<Image>,

    pub run_sword_layout: Handle<TextureAtlasLayout>,
    pub run_sword_texture: Handle<Image>,

    pub run_no_sword_layout: Handle<TextureAtlasLayout>,
    pub run_no_sword_texture: Handle<Image>,

    pub jump_sword_layout: Handle<TextureAtlasLayout>,
    pub jump_sword_texture: Handle<Image>,

    pub jump_no_sword_layout: Handle<TextureAtlasLayout>,
    pub jump_no_sword_texture: Handle<Image>,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_player_animations, spawn_player).chain())
            .add_systems(
                Update,
                (
                    player_input,
                    apply_gravity,
                    move_player,
                    select_animation,
                    animate_player,
                    update_player_flip,
                )
                    .chain(),
            );
    }
}

fn load_player_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let idle_sword_texture =
        asset_server.load("blue_knight/idle/action/blue_knight_action_idle.png");
    let idle_no_sword_texture = asset_server.load("blue_knight/idle/no_sword/blue_knight_idle.png");

    let run_sword_texture = asset_server.load("blue_knight/run/action/blue_knight_action_run.png");
    let run_no_sword_texture = asset_server.load("blue_knight/run/no_sword/blue_knight_run.png");

    let jump_sword_texture =
        asset_server.load("blue_knight/jump_stop/action/blue_knight_jump_action.png");
    let jump_no_sword_texture =
        asset_server.load("blue_knight/jump_stop/no_sword/blue_knight_jump_strip2.png");

    let idle_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 4, 1, None, None));
    let idle_no_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 4, 1, None, None));

    let run_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 6, 1, None, None));
    let run_no_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 6, 1, None, None));

    let jump_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 2, 1, None, None));
    let jump_no_sword_layout =
        atlas_layouts.add(TextureAtlasLayout::from_grid(FRAME_SIZE, 2, 1, None, None));

    commands.insert_resource(PlayerAnimationHandles {
        idle_sword_layout,
        idle_sword_texture,
        idle_no_sword_layout,
        idle_no_sword_texture,
        run_sword_layout,
        run_sword_texture,
        run_no_sword_layout,
        run_no_sword_texture,
        jump_sword_layout,
        jump_sword_texture,
        jump_no_sword_layout,
        jump_no_sword_texture,
    });
}

fn spawn_player(mut commands: Commands, anims: Res<PlayerAnimationHandles>) {
    commands.spawn((
        Sprite::from_atlas_image(
            anims.idle_sword_texture.clone(),
            TextureAtlas {
                layout: anims.idle_sword_layout.clone(),
                index: 0,
            },
        ),
        Transform::from_xyz(-450.0, GROUND_Y, 1.0).with_scale(Vec3::splat(PLAYER_SCALE)),
        Player,
        Velocity::default(),
        Facing(1.0),
        OnGround(true),
        HasSword(true),
        AnimationTimer(Timer::from_seconds(1.0 / IDLE_FPS, TimerMode::Repeating)),
        CurrentAnimation {
            state: PlayerAnimState::Idle,
            frame_count: 4,
        },
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

fn select_animation(
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

    *sprite = Sprite::from_atlas_image(texture, TextureAtlas { layout, index: 0 });

    timer.0 = Timer::from_seconds(1.0 / fps, TimerMode::Repeating);
    current.state = next_state;
    current.frame_count = frame_count;
}

fn animate_player(
    time: Res<Time>,
    mut query: Query<
        (
            &Velocity,
            &mut AnimationTimer,
            &CurrentAnimation,
            &mut Sprite,
        ),
        With<Player>,
    >,
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

fn update_player_flip(mut query: Query<(&Facing, &mut Transform), With<Player>>) {
    let Ok((facing, mut transform)) = query.single_mut() else {
        return;
    };

    transform.scale.x = PLAYER_SCALE * facing.0;
    transform.scale.y = PLAYER_SCALE;
}
