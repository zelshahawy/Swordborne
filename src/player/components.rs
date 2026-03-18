use bevy::prelude::*;

pub const GROUND_Y: f32 = -250.0;
pub const PLAYER_SPEED: f32 = 300.0;
pub const JUMP_VELOCITY: f32 = 500.0;
pub const GRAVITY: f32 = -1200.0;

pub const FRAME_SIZE: UVec2 = UVec2::new(24, 24);
pub const SLASH_FRAME_SIZE: UVec2 = UVec2::new(48, 48);

pub const PLAYER_SCALE: f32 = 4.0;
pub const PLAYER_HALF_WIDTH: f32 = 18.0;
pub const PLAYER_HEIGHT: f32 = 80.0;

pub const IDLE_FPS: f32 = 6.0;
pub const RUN_FPS: f32 = 10.0;
pub const JUMP_FPS: f32 = 8.0;
pub const SLASH_FPS: f32 = 12.0;

pub const SLASH_FRAMES: usize = 4;

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
    Slash,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum PlayerActionState {
    #[default]
    None,
    Slash,
}

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub struct ActionTimer(pub Timer);

#[derive(Component)]
pub struct CurrentAnimation {
    pub state: PlayerAnimState,
    pub frame_count: usize,
    pub with_sword: bool,
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

    pub slash_sword_layout: Handle<TextureAtlasLayout>,
    pub slash_sword_texture: Handle<Image>,
}
