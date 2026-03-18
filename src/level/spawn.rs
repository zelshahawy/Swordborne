use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::player::{GROUND_Y, PlayerAnimationHandles, spawn::spawn_player_entity};
use crate::state::{CampaignState, LevelId, PlayerProfile};
use crate::sword::{SwordState, SwordVisualHandles, spawn::spawn_sword_entity};

use super::assets::LevelArtHandles;
use super::components::{
    LevelBounds, LevelEntity, TrainingCrate, TrainingDoor, TutorialMarker, WizardAnimationFrame,
    WizardAnimationTimer, WizardNpc,
};
use super::logic::wizard_scale;
use super::{
    LEVEL_ONE_CRATE_X, LEVEL_ONE_DOOR_X, LEVEL_ONE_PLAYER_START_X, LEVEL_ONE_SWORD_X,
    LEVEL_ONE_TUTORIAL_X, LEVEL_ONE_WIZARD_X, ROOM_PLAYER_LEFT_X, ROOM_PLAYER_RIGHT_X,
    ROOM_WALL_LEFT_X, ROOM_WALL_RIGHT_X, TILE_SCALE, TILE_WORLD_SIZE,
};

pub(super) fn spawn_current_level(
    mut commands: Commands,
    art: Res<LevelArtHandles>,
    player_anims: Res<PlayerAnimationHandles>,
    sword_visuals: Res<SwordVisualHandles>,
    campaign: Res<CampaignState>,
    profile: Res<PlayerProfile>,
) {
    spawn_level_scene(
        &mut commands,
        &art,
        &player_anims,
        &sword_visuals,
        &campaign,
        &profile,
    );
}

pub(super) fn despawn_level_entities(
    mut commands: Commands,
    query: Query<Entity, With<LevelEntity>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(super) fn spawn_level_scene(
    commands: &mut Commands,
    art: &LevelArtHandles,
    player_anims: &PlayerAnimationHandles,
    sword_visuals: &SwordVisualHandles,
    campaign: &CampaignState,
    profile: &PlayerProfile,
) {
    match campaign.current_level {
        LevelId::LevelOne => spawn_level_one(commands, art, player_anims, sword_visuals),
        LevelId::LevelTwo => spawn_level_two(commands, art, player_anims, sword_visuals, profile),
    }
}

fn spawn_level_one(
    commands: &mut Commands,
    art: &LevelArtHandles,
    player_anims: &PlayerAnimationHandles,
    sword_visuals: &SwordVisualHandles,
) {
    commands.insert_resource(LevelBounds {
        wall_left_x: ROOM_WALL_LEFT_X,
        wall_right_x: ROOM_WALL_RIGHT_X,
        player_left_x: LEVEL_ONE_PLAYER_START_X,
        player_right_x: ROOM_PLAYER_RIGHT_X,
    });

    spawn_room_shell(commands, art, "LEVEL 1");

    let player = spawn_player_entity(
        commands,
        player_anims,
        Vec3::new(LEVEL_ONE_PLAYER_START_X, GROUND_Y, 5.0),
        false,
    );
    commands.entity(player).insert(LevelEntity);

    let sword = spawn_sword_entity(
        commands,
        sword_visuals,
        Vec3::new(LEVEL_ONE_SWORD_X, GROUND_Y, 4.0),
        SwordState::Grounded,
    );
    commands.entity(sword).insert(LevelEntity);

    spawn_wizard(commands, art, Vec3::new(LEVEL_ONE_WIZARD_X, GROUND_Y, 4.0));
    spawn_tutorial_marker(
        commands,
        art,
        Vec3::new(LEVEL_ONE_TUTORIAL_X, GROUND_Y, 4.0),
    );
    spawn_training_crate(commands, art, Vec3::new(LEVEL_ONE_CRATE_X, GROUND_Y, 4.0));
    spawn_training_door(
        commands,
        art,
        Vec3::new(LEVEL_ONE_DOOR_X, GROUND_Y, 4.0),
        false,
    );
}

fn spawn_level_two(
    commands: &mut Commands,
    art: &LevelArtHandles,
    player_anims: &PlayerAnimationHandles,
    sword_visuals: &SwordVisualHandles,
    profile: &PlayerProfile,
) {
    commands.insert_resource(LevelBounds {
        wall_left_x: ROOM_WALL_LEFT_X,
        wall_right_x: ROOM_WALL_RIGHT_X,
        player_left_x: ROOM_PLAYER_LEFT_X,
        player_right_x: ROOM_PLAYER_RIGHT_X,
    });

    spawn_room_shell(commands, art, "LEVEL 2");

    let player = spawn_player_entity(
        commands,
        player_anims,
        Vec3::new(-500.0, GROUND_Y, 5.0),
        true,
    );
    commands.entity(player).insert(LevelEntity);

    let sword = spawn_sword_entity(
        commands,
        sword_visuals,
        Vec3::new(-500.0, GROUND_Y, 4.0),
        SwordState::Equipped,
    );
    commands.entity(sword).insert(LevelEntity);

    let knight_name = if profile.name.is_empty() {
        "Knight"
    } else {
        profile.name.as_str()
    };

    commands.spawn((
        LevelEntity,
        Text2d::new(format!(
            "Well done, {knight_name}.\nLevel 2 is ready for the next trial."
        )),
        TextFont::from_font_size(34.0),
        TextColor(Color::srgb(0.97, 0.92, 0.72)),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, 40.0, 10.0),
    ));
}

fn spawn_room_shell(commands: &mut Commands, art: &LevelArtHandles, level_label: &str) {
    commands.spawn((
        LevelEntity,
        Sprite::from_color(Color::srgb(0.04, 0.05, 0.08), Vec2::new(1400.0, 900.0)),
        Transform::from_xyz(0.0, 0.0, -20.0),
    ));

    commands.spawn((
        LevelEntity,
        Sprite::from_color(Color::srgb(0.1, 0.13, 0.18), Vec2::new(1200.0, 520.0)),
        Transform::from_xyz(0.0, -20.0, -18.0),
    ));

    for tile_index in 0..20 {
        let x = -608.0 + tile_index as f32 * TILE_WORLD_SIZE;
        spawn_centered_tile(
            commands,
            art.floor.clone(),
            Vec3::new(x, GROUND_Y - 32.0, 0.0),
        );
    }

    let top_y = GROUND_Y + 288.0;
    spawn_centered_tile(
        commands,
        art.wall_top_left.clone(),
        Vec3::new(-608.0, top_y, 0.0),
    );

    for tile_index in 1..19 {
        let x = -608.0 + tile_index as f32 * TILE_WORLD_SIZE;
        spawn_centered_tile(commands, art.wall_top_mid.clone(), Vec3::new(x, top_y, 0.0));
    }

    spawn_centered_tile(
        commands,
        art.wall_top_right.clone(),
        Vec3::new(608.0, top_y, 0.0),
    );

    for row in 0..5 {
        let y = GROUND_Y + 32.0 + row as f32 * TILE_WORLD_SIZE;
        spawn_centered_tile(commands, art.wall_mid.clone(), Vec3::new(-608.0, y, 0.0));
        spawn_centered_tile(commands, art.wall_mid.clone(), Vec3::new(608.0, y, 0.0));
    }

    spawn_bottom_anchored_sprite(
        commands,
        art.column_wall.clone(),
        Vec3::new(-472.0, GROUND_Y, 1.0),
        TILE_SCALE,
    );
    spawn_bottom_anchored_sprite(
        commands,
        art.column_wall.clone(),
        Vec3::new(472.0, GROUND_Y, 1.0),
        TILE_SCALE,
    );
    spawn_bottom_anchored_sprite(
        commands,
        art.banner_blue.clone(),
        Vec3::new(-180.0, GROUND_Y + 180.0, 1.0),
        TILE_SCALE,
    );
    spawn_bottom_anchored_sprite(
        commands,
        art.banner_red.clone(),
        Vec3::new(220.0, GROUND_Y + 180.0, 1.0),
        TILE_SCALE,
    );

    commands.spawn((
        LevelEntity,
        Text2d::new(level_label.to_string()),
        TextFont::from_font_size(28.0),
        TextColor(Color::srgb(0.81, 0.85, 0.92)),
        Transform::from_xyz(0.0, top_y - 30.0, 2.0),
    ));
}

fn spawn_wizard(commands: &mut Commands, art: &LevelArtHandles, position: Vec3) {
    commands.spawn((
        LevelEntity,
        WizardNpc,
        WizardAnimationFrame::default(),
        WizardAnimationTimer(Timer::from_seconds(0.16, TimerMode::Repeating)),
        Sprite::from_image(art.wizard_idle_frames[0].clone()),
        Anchor::BOTTOM_CENTER,
        Transform::from_translation(position).with_scale(Vec3::splat(wizard_scale())),
    ));
}

fn spawn_tutorial_marker(commands: &mut Commands, art: &LevelArtHandles, position: Vec3) {
    spawn_bottom_anchored_sprite(commands, art.tutorial_base.clone(), position, TILE_SCALE);

    commands.spawn((
        LevelEntity,
        TutorialMarker,
        Transform::from_translation(position),
        GlobalTransform::default(),
    ));

    commands.spawn((
        LevelEntity,
        Text2d::new("!".to_string()),
        TextFont::from_font_size(54.0),
        TextColor(Color::srgb(0.99, 0.86, 0.34)),
        Transform::from_xyz(position.x, position.y + 120.0, position.z + 1.0),
    ));
}

fn spawn_training_crate(commands: &mut Commands, art: &LevelArtHandles, position: Vec3) {
    commands.spawn((
        LevelEntity,
        TrainingCrate,
        Sprite::from_image(art.crate_texture.clone()),
        Anchor::BOTTOM_CENTER,
        Transform::from_translation(position).with_scale(Vec3::splat(TILE_SCALE)),
    ));
}

fn spawn_training_door(commands: &mut Commands, art: &LevelArtHandles, position: Vec3, open: bool) {
    commands.spawn((
        LevelEntity,
        TrainingDoor { open },
        if open {
            Sprite::from_image(art.door_open.clone())
        } else {
            Sprite::from_image(art.door_closed.clone())
        },
        Anchor::BOTTOM_CENTER,
        Transform::from_translation(position).with_scale(Vec3::splat(TILE_SCALE)),
    ));

    spawn_bottom_anchored_sprite(
        commands,
        art.column_wall.clone(),
        position + Vec3::new(-52.0, 0.0, -0.1),
        TILE_SCALE,
    );
    spawn_bottom_anchored_sprite(
        commands,
        art.column_wall.clone(),
        position + Vec3::new(52.0, 0.0, -0.1),
        TILE_SCALE,
    );
}

fn spawn_centered_tile(commands: &mut Commands, texture: Handle<Image>, position: Vec3) {
    commands.spawn((
        LevelEntity,
        Sprite::from_image(texture),
        Transform::from_translation(position).with_scale(Vec3::splat(TILE_SCALE)),
    ));
}

fn spawn_bottom_anchored_sprite(
    commands: &mut Commands,
    texture: Handle<Image>,
    position: Vec3,
    scale: f32,
) {
    commands.spawn((
        LevelEntity,
        Sprite::from_image(texture),
        Anchor::BOTTOM_CENTER,
        Transform::from_translation(position).with_scale(Vec3::splat(scale)),
    ));
}
