use bevy::prelude::*;

pub const BOSS_MAX_HP: i32 = 5;
pub const BOSS_HIT_RADIUS: f32 = 60.0;
pub const BOSS_INVINCIBILITY_SECS: f32 = 0.55;
pub const BOSS_CHASE_SPEED: f32 = 185.0;
pub const BOSS_CHASE_SPEED_P2: f32 = 320.0;
pub const BOSS_CHARGE_WINDUP: f32 = 0.36;
pub const BOSS_CHARGE_WINDUP_P2: f32 = 0.13;
pub const BOSS_CHARGE_SPEED: f32 = 730.0;
pub const BOSS_CHARGE_SPEED_P2: f32 = 1020.0;
pub const BOSS_CHARGE_DURATION: f32 = 0.52;
pub const BOSS_CHARGE_TRIGGER_DIST: f32 = 300.0;
pub const BOSS_STAGGER_DURATION: f32 = 0.28;
pub const BOSS_DISARM_DIST: f32 = 52.0;

#[derive(Component)]
pub struct Boss;

#[derive(Component)]
pub struct BossHp {
    pub current: i32,
    pub max: i32, // used for HP bar ratio
}

#[derive(Component, PartialEq, Clone)]
pub enum BossPhase {
    Chase,
    Windup { dir: Vec2, timer: f32 },
    Charge { vel: Vec2, timer: f32 },
    Stagger { timer: f32 },
    Dead,
}

#[derive(Component)]
pub struct BossInvincible(pub f32);

#[derive(Component)]
pub struct BossAnimationTimer(pub Timer);

#[derive(Component, Default)]
pub struct BossAnimationFrame(pub usize);

#[derive(Component)]
pub struct BossHpFill;

#[derive(Component)]
pub struct BossDefeatedText;

#[derive(Component)]
pub struct PlayerHpHeart(pub usize);
