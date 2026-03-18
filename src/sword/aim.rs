use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::PrimaryWindow;

use crate::level::{LevelBounds, SwordBlocker};
use crate::player::{Facing, HasSword, Player, PlayerActionState};
use crate::sword::{
    Sword, SwordAimGuide, SwordAimReticle, SwordAimState, SwordAnimationTimer, SwordFlight,
    SwordState, SwordVelocity, SwordVisualHandles,
};

use super::logic::{
    FLYING_SWORD_HALF_SIZE, MAX_STRAIGHT_THROW_DISTANCE, RayHit, THROW_SPEED, raycast_against_aabb,
    raycast_against_bounds, sword_launch_origin,
};

const AIM_GUIDE_WIDTH: f32 = 12.0;
const AIM_GUIDE_MIN_LENGTH: f32 = 18.0;
const AIM_GUIDE_ALPHA: f32 = 0.78;
const AIM_RETICLE_SIZE: f32 = 18.0;

type PlayerAimQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        &'static mut Facing,
        &'static mut HasSword,
        &'static PlayerActionState,
    ),
    (With<Player>, Without<Sword>),
>;

type SwordAimQuery<'w, 's> = Query<
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

type GuideQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Visibility,
        &'static mut Sprite,
        &'static mut Transform,
    ),
    (
        With<SwordAimGuide>,
        Without<SwordAimReticle>,
        Without<Player>,
        Without<Sword>,
        Without<SwordBlocker>,
    ),
>;

type ReticleQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Visibility,
        &'static mut Sprite,
        &'static mut Transform,
    ),
    (
        With<SwordAimReticle>,
        Without<SwordAimGuide>,
        Without<Player>,
        Without<Sword>,
        Without<SwordBlocker>,
    ),
>;

pub fn spawn_aim_preview(mut commands: Commands, mut aim: ResMut<SwordAimState>) {
    aim.reset();

    commands.spawn((
        SwordAimGuide,
        Visibility::Hidden,
        Sprite {
            color: aim_guide_color(),
            custom_size: Some(Vec2::new(0.0, AIM_GUIDE_WIDTH)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 6.5),
    ));

    commands.spawn((
        SwordAimReticle,
        Visibility::Hidden,
        Sprite {
            color: aim_guide_color(),
            custom_size: Some(Vec2::splat(AIM_RETICLE_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 6.6)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
    ));
}

pub fn despawn_aim_preview(
    mut commands: Commands,
    mut aim: ResMut<SwordAimState>,
    guide_query: Query<Entity, With<SwordAimGuide>>,
    reticle_query: Query<Entity, With<SwordAimReticle>>,
) {
    aim.reset();

    for entity in &guide_query {
        commands.entity(entity).despawn();
    }

    for entity in &reticle_query {
        commands.entity(entity).despawn();
    }
}

pub fn begin_sword_aim(
    mouse: Res<ButtonInput<MouseButton>>,
    mut aim: ResMut<SwordAimState>,
    player_query: PlayerAimQuery,
    sword_query: Query<&SwordState, With<Sword>>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok((_, _, has_sword, action_state)) = player_query.single() else {
        return;
    };
    let Ok(sword_state) = sword_query.single() else {
        return;
    };

    if !has_sword.0
        || *action_state != PlayerActionState::None
        || *sword_state != SwordState::Equipped
    {
        return;
    }

    aim.active = true;
}

pub fn update_sword_aim_preview(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    bounds: Res<LevelBounds>,
    blocker_query: Query<(&Transform, &SwordBlocker), Without<Sword>>,
    mut aim: ResMut<SwordAimState>,
    mut player_query: PlayerAimQuery,
    sword_query: Query<&SwordState, With<Sword>>,
    mut guide_query: GuideQuery,
    mut reticle_query: ReticleQuery,
) {
    let Ok((mut guide_visibility, mut guide_sprite, mut guide_transform)) =
        guide_query.single_mut()
    else {
        return;
    };
    let Ok((mut reticle_visibility, mut reticle_sprite, mut reticle_transform)) =
        reticle_query.single_mut()
    else {
        return;
    };

    let Ok((player_transform, mut facing, has_sword, action_state)) = player_query.single_mut()
    else {
        hide_preview(&mut aim, &mut guide_visibility, &mut reticle_visibility);
        return;
    };
    let Ok(sword_state) = sword_query.single() else {
        hide_preview(&mut aim, &mut guide_visibility, &mut reticle_visibility);
        return;
    };

    if !aim.active
        || !has_sword.0
        || *action_state != PlayerActionState::None
        || *sword_state != SwordState::Equipped
    {
        hide_preview(&mut aim, &mut guide_visibility, &mut reticle_visibility);
        return;
    }

    let Some(cursor_world) = cursor_world_position(&windows, &camera_query) else {
        hide_preview(&mut aim, &mut guide_visibility, &mut reticle_visibility);
        return;
    };

    let mut raw_direction = cursor_world - player_transform.translation.truncate();
    if raw_direction.length_squared() < 0.001 {
        raw_direction = Vec2::new(facing.0, 0.0);
    }
    let raw_direction = raw_direction.normalize_or_zero();

    if raw_direction.x.abs() > 0.15 {
        facing.0 = raw_direction.x.signum();
    }

    let origin = sword_launch_origin(player_transform, raw_direction).truncate();
    let direction = (cursor_world - origin).normalize_or_zero();

    if direction.length_squared() < 0.001 {
        hide_preview(&mut aim, &mut guide_visibility, &mut reticle_visibility);
        return;
    }

    let (preview_end, blocked) = preview_target(origin, direction, &bounds, &blocker_query);
    let segment = preview_end - origin;
    let segment_length = segment.length();

    aim.origin = origin;
    aim.direction = direction;
    aim.preview_end = preview_end;
    aim.blocked = blocked;

    *guide_visibility = Visibility::Visible;
    *reticle_visibility = Visibility::Visible;

    guide_sprite.color = if blocked {
        aim_blocked_color()
    } else {
        aim_guide_color()
    };
    guide_sprite.custom_size = Some(Vec2::new(
        segment_length.max(AIM_GUIDE_MIN_LENGTH),
        AIM_GUIDE_WIDTH,
    ));
    guide_transform.translation = ((origin + preview_end) * 0.5).extend(6.5);
    guide_transform.rotation = Quat::from_rotation_z(segment.y.atan2(segment.x));

    reticle_sprite.color = if blocked {
        aim_blocked_color()
    } else {
        aim_guide_color()
    };
    reticle_transform.translation = preview_end.extend(6.6);
}

pub fn release_sword_aim(
    mouse: Res<ButtonInput<MouseButton>>,
    sword_visuals: Res<SwordVisualHandles>,
    mut aim: ResMut<SwordAimState>,
    mut player_query: PlayerAimQuery,
    mut sword_query: SwordAimQuery,
    mut guide_query: GuideQuery,
    mut reticle_query: ReticleQuery,
) {
    if !mouse.just_released(MouseButton::Right) {
        return;
    }

    let Ok((mut guide_visibility, _, _)) = guide_query.single_mut() else {
        return;
    };
    let Ok((mut reticle_visibility, _, _)) = reticle_query.single_mut() else {
        return;
    };

    let Ok((player_transform, _, mut has_sword, action_state)) = player_query.single_mut() else {
        hide_preview(&mut aim, &mut guide_visibility, &mut reticle_visibility);
        return;
    };

    if !aim.active
        || !has_sword.0
        || *action_state != PlayerActionState::None
        || aim.direction.length_squared() < 0.001
    {
        hide_preview(&mut aim, &mut guide_visibility, &mut reticle_visibility);
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

        let origin = sword_launch_origin(player_transform, aim.direction);
        sword_transform.translation = origin;
        sword_transform.rotation = Quat::IDENTITY;

        sword_velocity.x = aim.direction.x * THROW_SPEED;
        sword_velocity.y = aim.direction.y * THROW_SPEED;

        *sprite = Sprite::from_atlas_image(
            sword_visuals.spinning_texture.clone(),
            TextureAtlas {
                layout: sword_visuals.spinning_layout.clone(),
                index: 0,
            },
        );
        *anchor = Anchor::CENTER;
        flight.launch_position = origin.truncate();
        flight.last_trail_position = flight.launch_position;
        flight.trail_timer.reset();
        anim_timer.0.reset();
        break;
    }

    hide_preview(&mut aim, &mut guide_visibility, &mut reticle_visibility);
}

fn preview_target(
    origin: Vec2,
    direction: Vec2,
    bounds: &LevelBounds,
    blocker_query: &Query<(&Transform, &SwordBlocker), Without<Sword>>,
) -> (Vec2, bool) {
    let mut best_hit =
        raycast_against_bounds(origin, direction, MAX_STRAIGHT_THROW_DISTANCE, bounds);

    for (transform, blocker) in blocker_query.iter() {
        take_closer_hit(
            &mut best_hit,
            raycast_against_aabb(
                origin,
                direction,
                MAX_STRAIGHT_THROW_DISTANCE,
                transform.translation.truncate(),
                blocker.half_extents + Vec2::splat(FLYING_SWORD_HALF_SIZE - 2.0),
            ),
        );
    }

    if let Some(hit) = best_hit {
        (hit.point, true)
    } else {
        (origin + direction * MAX_STRAIGHT_THROW_DISTANCE, false)
    }
}

fn cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) -> Option<Vec2> {
    let Ok(window) = windows.single() else {
        return None;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return None;
    };
    let cursor = window.cursor_position()?;

    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

fn take_closer_hit(best_hit: &mut Option<RayHit>, candidate: Option<RayHit>) {
    let Some(candidate) = candidate else {
        return;
    };

    if best_hit.is_none_or(|current| candidate.distance < current.distance) {
        *best_hit = Some(candidate);
    }
}

fn hide_preview(
    aim: &mut SwordAimState,
    guide_visibility: &mut Visibility,
    reticle_visibility: &mut Visibility,
) {
    aim.reset();
    *guide_visibility = Visibility::Hidden;
    *reticle_visibility = Visibility::Hidden;
}

fn aim_guide_color() -> Color {
    Color::srgba(0.95, 0.97, 1.0, AIM_GUIDE_ALPHA)
}

fn aim_blocked_color() -> Color {
    Color::srgba(0.98, 0.84, 0.42, 0.82)
}
