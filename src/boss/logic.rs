use bevy::prelude::*;

use crate::boss::components::*;
use crate::fonts::GameFonts;
use crate::level::{LevelArtHandles, LevelBounds, LevelEntity};
use crate::player::{GROUND_Y, HasSword, Player};
use crate::state::{FadePhase, FadeState, PlayerHealth};
use crate::sword::{Sword, SwordState};

const BOSS_MIN_Y: f32 = GROUND_Y + 30.0;
const BOSS_MAX_Y_OFFSET: f32 = 80.0;

fn boss_tint(p2: bool) -> Color {
    if p2 { Color::srgb(1.0, 0.12, 0.12) } else { Color::srgb(1.0, 0.28, 0.28) }
}

pub(crate) fn tick_boss(
    time: Res<Time>,
    art: Res<LevelArtHandles>,
    bounds: Res<LevelBounds>,
    player_query: Query<&Transform, (With<Player>, Without<Boss>)>,
    mut boss_query: Query<(
        &mut Transform,
        &mut Sprite,
        &mut BossPhase,
        &mut BossInvincible,
        &mut BossAnimationTimer,
        &mut BossAnimationFrame,
        &BossHp,
    ), With<Boss>>,
) {
    let Ok(player_tf) = player_query.single() else { return; };
    let Ok((mut tf, mut sprite, mut phase, mut inv, mut anim_timer, mut anim_frame, hp)) =
        boss_query.single_mut()
    else {
        return;
    };

    let dt = time.delta_secs();
    let is_p2 = hp.current <= 2;

    inv.0 = (inv.0 - dt).max(0.0);

    sprite.color = if inv.0 > 0.0 && (inv.0 * 14.0) as i32 % 2 == 0 {
        Color::WHITE
    } else {
        boss_tint(is_p2)
    };

    anim_timer.0.tick(time.delta());
    if anim_timer.0.just_finished() {
        anim_frame.0 = (anim_frame.0 + 1) % art.wizard_idle_frames.len();
        sprite.image = art.wizard_idle_frames[anim_frame.0].clone();
    }

    let boss_max_y = bounds.ceiling_y - BOSS_MAX_Y_OFFSET;

    match phase.clone() {
        BossPhase::Dead => {}

        BossPhase::Stagger { timer } => {
            let t = timer - dt;
            *phase = if t <= 0.0 { BossPhase::Chase } else { BossPhase::Stagger { timer: t } };
        }

        BossPhase::Windup { dir, timer } => {
            let t = timer - dt;
            if t <= 0.0 {
                let speed = if is_p2 { BOSS_CHARGE_SPEED_P2 } else { BOSS_CHARGE_SPEED };
                *phase = BossPhase::Charge { vel: dir * speed, timer: BOSS_CHARGE_DURATION };
            } else {
                *phase = BossPhase::Windup { dir, timer: t };
            }
        }

        BossPhase::Charge { vel, timer } => {
            tf.translation.x = (tf.translation.x + vel.x * dt)
                .clamp(bounds.wall_left_x + 44.0, bounds.wall_right_x - 44.0);
            tf.translation.y = (tf.translation.y + vel.y * dt)
                .clamp(BOSS_MIN_Y, boss_max_y);
            sprite.flip_x = vel.x < 0.0;
            let t = timer - dt;
            *phase = if t <= 0.0 {
                BossPhase::Chase
            } else {
                BossPhase::Charge { vel, timer: t }
            };
        }

        BossPhase::Chase => {
            let dp = Vec2::new(
                player_tf.translation.x - tf.translation.x,
                player_tf.translation.y - tf.translation.y,
            );
            let speed = if is_p2 { BOSS_CHASE_SPEED_P2 } else { BOSS_CHASE_SPEED };
            let dir = dp.normalize_or_zero();
            tf.translation.x = (tf.translation.x + dir.x * speed * dt)
                .clamp(bounds.wall_left_x + 44.0, bounds.wall_right_x - 44.0);
            tf.translation.y = (tf.translation.y + dir.y * speed * dt)
                .clamp(BOSS_MIN_Y, boss_max_y);
            sprite.flip_x = dp.x < 0.0;

            if dp.length() < BOSS_CHARGE_TRIGGER_DIST {
                let windup = if is_p2 { BOSS_CHARGE_WINDUP_P2 } else { BOSS_CHARGE_WINDUP };
                *phase = BossPhase::Windup { dir, timer: windup };
            }
        }
    }
}

pub(crate) fn boss_take_damage(
    mut boss_query: Query<
        (&Transform, &mut BossHp, &mut BossPhase, &mut BossInvincible),
        With<Boss>,
    >,
    sword_query: Query<(&Transform, &SwordState), With<Sword>>,
) {
    let Ok((boss_tf, mut hp, mut phase, mut inv)) = boss_query.single_mut() else { return; };
    if *phase == BossPhase::Dead || inv.0 > 0.0 {
        return;
    }

    for (sword_tf, sword_state) in &sword_query {
        if *sword_state != SwordState::Flying {
            continue;
        }
        if sword_tf.translation.distance(boss_tf.translation) <= BOSS_HIT_RADIUS {
            hp.current -= 1;
            inv.0 = BOSS_INVINCIBILITY_SECS;
            *phase = if hp.current <= 0 {
                hp.current = 0;
                BossPhase::Dead
            } else {
                BossPhase::Stagger { timer: BOSS_STAGGER_DURATION }
            };
            break;
        }
    }
}

pub(crate) fn boss_disarm_player(
    boss_query: Query<(&Transform, &BossPhase), With<Boss>>,
    mut player_query: Query<(&Transform, &mut HasSword), (With<Player>, Without<Boss>)>,
    mut sword_query: Query<
        (&mut Transform, &mut SwordState, &mut Visibility),
        (With<Sword>, Without<Boss>, Without<Player>),
    >,
) {
    let Ok((boss_tf, phase)) = boss_query.single() else { return; };
    let vel_x = match phase {
        BossPhase::Charge { vel, .. } => vel.x,
        _ => return,
    };
    let Ok((player_tf, mut has_sword)) = player_query.single_mut() else { return; };
    if !has_sword.0 {
        return;
    }
    if boss_tf.translation.distance(player_tf.translation) > BOSS_DISARM_DIST {
        return;
    }

    has_sword.0 = false;
    let fling_x = (player_tf.translation.x - vel_x.signum() * 300.0).clamp(-680.0, 680.0);
    for (mut sword_tf, mut state, mut vis) in &mut sword_query {
        if *state == SwordState::Equipped {
            *state = SwordState::Grounded;
            *vis = Visibility::Visible;
            sword_tf.translation.x = fling_x;
            sword_tf.translation.y = GROUND_Y;
            break;
        }
    }
}

pub(crate) fn boss_hit_player(
    boss_query: Query<(&Transform, &BossPhase), With<Boss>>,
    player_query: Query<&Transform, With<Player>>,
    mut player_health: ResMut<PlayerHealth>,
    mut fade_state: ResMut<FadeState>,
) {
    let Ok((boss_tf, phase)) = boss_query.single() else { return; };
    if !matches!(phase, BossPhase::Charge { .. }) {
        return;
    }
    if player_health.invincibility_timer > 0.0 {
        return;
    }
    let Ok(player_tf) = player_query.single() else { return; };
    if boss_tf.translation.distance(player_tf.translation) > BOSS_DISARM_DIST {
        return;
    }

    player_health.current -= 1;
    player_health.invincibility_timer = 1.2;

    if player_health.current <= 0 && fade_state.phase == FadePhase::Idle {
        fade_state.phase = FadePhase::FadeOut(0.0);
        fade_state.trigger_restart = true;
    }
}

pub(crate) fn tick_player_invincibility(
    time: Res<Time>,
    mut player_health: ResMut<PlayerHealth>,
    mut player_query: Query<&mut Sprite, With<Player>>,
) {
    let dt = time.delta_secs();
    if player_health.invincibility_timer > 0.0 {
        player_health.invincibility_timer = (player_health.invincibility_timer - dt).max(0.0);
    }
    let Ok(mut sprite) = player_query.single_mut() else { return; };
    sprite.color = if player_health.invincibility_timer > 0.0
        && (player_health.invincibility_timer * 12.0) as i32 % 2 == 0
    {
        Color::srgb(1.0, 0.2, 0.2)
    } else {
        Color::WHITE
    };
}

pub(crate) fn sync_boss_hp_bar(
    boss_query: Query<&BossHp, With<Boss>>,
    mut fill_query: Query<&mut Node, With<BossHpFill>>,
) {
    let Ok(hp) = boss_query.single() else { return; };
    let Ok(mut fill_node) = fill_query.single_mut() else { return; };
    let ratio = (hp.current as f32 / hp.max as f32).max(0.0);
    fill_node.width = Val::Percent(ratio * 100.0);
}

pub(crate) fn handle_boss_defeat(
    mut commands: Commands,
    boss_query: Query<(Entity, &BossPhase), With<Boss>>,
    mut text_query: Query<&mut Visibility, With<BossDefeatedText>>,
) {
    let Ok((boss_entity, phase)) = boss_query.single() else { return; };
    if *phase != BossPhase::Dead {
        return;
    }
    commands.entity(boss_entity).despawn();
    for mut vis in &mut text_query {
        *vis = Visibility::Visible;
    }
}

pub(crate) fn sync_player_hp_display(
    player_health: Res<PlayerHealth>,
    mut query: Query<(&PlayerHpHeart, &mut BackgroundColor)>,
) {
    if !player_health.is_changed() {
        return;
    }
    for (heart, mut color) in &mut query {
        *color = if heart.0 < player_health.current as usize {
            BackgroundColor(Color::srgb(0.88, 0.12, 0.12))
        } else {
            BackgroundColor(Color::srgb(0.18, 0.07, 0.07))
        };
    }
}

pub(crate) fn spawn_player_hp_ui(commands: &mut Commands, fonts: &GameFonts) {
    commands
        .spawn((
            LevelEntity,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                LevelEntity,
                Text::new("LIVES"),
                TextFont { font: fonts.pixel_bold.clone(), font_size: 11.0, ..default() },
                TextColor(Color::srgb(0.75, 0.28, 0.28)),
            ));
            parent
                .spawn((
                    LevelEntity,
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        ..default()
                    },
                ))
                .with_children(|row| {
                    for i in 0..3usize {
                        row.spawn((
                            LevelEntity,
                            PlayerHpHeart(i),
                            Node {
                                width: Val::Px(16.0),
                                height: Val::Px(16.0),
                                border_radius: BorderRadius::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.88, 0.12, 0.12)),
                        ));
                    }
                });
        });
}

pub(crate) fn spawn_boss_hp_bar(commands: &mut Commands, fonts: &GameFonts) {
    commands
        .spawn((
            LevelEntity,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                LevelEntity,
                Text::new("THE DARK WIZARD"),
                TextFont { font: fonts.pixel_bold.clone(), font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.95, 0.28, 0.28)),
            ));
            parent
                .spawn((
                    LevelEntity,
                    Node {
                        width: Val::Px(320.0),
                        height: Val::Px(14.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.92)),
                    BorderColor::all(Color::srgba(0.7, 0.1, 0.1, 0.8)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        LevelEntity,
                        BossHpFill,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.82, 0.07, 0.07)),
                    ));
                });
        });
}
