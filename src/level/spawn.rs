use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::fonts::GameFonts;
use crate::level::{
    BreakableCrate, CrateReward, LEVEL_ONE_CRATE_X, LEVEL_ONE_DOOR_X, LEVEL_ONE_PLAYER_START_X,
    LEVEL_ONE_SWORD_X, LEVEL_ONE_TUTORIAL_X, LEVEL_ONE_WIZARD_X, LEVEL_TWO_CRATE_X,
    LEVEL_TWO_HINT_X, LEVEL_TWO_HINT_Y, LEVEL_TWO_PLAYER_START_X, LEVEL_TWO_SHELF_TOP_Y,
    LevelArtHandles, LevelBounds, LevelEntity, LevelTwoCompletionText, ROOM_CEILING_Y,
    ROOM_PLAYER_LEFT_X, ROOM_PLAYER_RIGHT_X, ROOM_WALL_LEFT_X, ROOM_WALL_RIGHT_X, SwordBlocker,
    TILE_SCALE, TILE_WORLD_SIZE, TrainingDoor, TutorialMarker, WizardAnimationFrame,
    WizardAnimationTimer, WizardNpc, frame_level_camera, spawn_room_shell, wizard_scale,
};
use crate::player::{GROUND_Y, PlayerAnimationHandles, spawn::spawn_player_entity};
use crate::state::{CampaignState, LevelId, PlayerProfile};
use crate::sword::{SwordState, SwordVisualHandles, spawn::spawn_sword_entity};

pub(crate) fn spawn_current_level(
    mut commands: Commands,
    art: Res<LevelArtHandles>,
    player_anims: Res<PlayerAnimationHandles>,
    sword_visuals: Res<SwordVisualHandles>,
    campaign: Res<CampaignState>,
    profile: Res<PlayerProfile>,
    fonts: Res<GameFonts>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let bounds = level_bounds_for(campaign.current_level);
    frame_level_camera(
        &mut camera_query,
        Some(&window_query),
        Some(bounds),
        Some(level_camera_focus_x(campaign.current_level)),
    );
    spawn_level_scene(
        &mut commands,
        &art,
        &fonts,
        &player_anims,
        &sword_visuals,
        &campaign,
        &profile,
    );
}

pub(crate) fn despawn_level_entities(
    mut commands: Commands,
    query: Query<Entity, With<LevelEntity>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn spawn_level_scene(
    commands: &mut Commands,
    art: &LevelArtHandles,
    fonts: &GameFonts,
    player_anims: &PlayerAnimationHandles,
    sword_visuals: &SwordVisualHandles,
    campaign: &CampaignState,
    profile: &PlayerProfile,
) {
    match campaign.current_level {
        LevelId::LevelOne => spawn_level_one(commands, art, fonts, player_anims, sword_visuals),
        LevelId::LevelTwo => {
            spawn_level_two(commands, art, fonts, player_anims, sword_visuals, profile)
        }
    }
}

fn spawn_level_one(
    commands: &mut Commands,
    art: &LevelArtHandles,
    fonts: &GameFonts,
    player_anims: &PlayerAnimationHandles,
    sword_visuals: &SwordVisualHandles,
) {
    commands.insert_resource(level_bounds_for(LevelId::LevelOne));

    spawn_room_shell(commands, art, fonts, "LEVEL 1");

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
        fonts,
        Vec3::new(LEVEL_ONE_TUTORIAL_X, GROUND_Y, 4.0),
    );
    spawn_breakable_crate(
        commands,
        art,
        Vec3::new(LEVEL_ONE_CRATE_X, GROUND_Y, 4.0),
        CrateReward::OpenTrainingDoor,
    );
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
    fonts: &GameFonts,
    player_anims: &PlayerAnimationHandles,
    sword_visuals: &SwordVisualHandles,
    profile: &PlayerProfile,
) {
    commands.insert_resource(level_bounds_for(LevelId::LevelTwo));

    spawn_room_shell(commands, art, fonts, "LEVEL 2");

    let player = spawn_player_entity(
        commands,
        player_anims,
        Vec3::new(LEVEL_TWO_PLAYER_START_X, GROUND_Y, 5.0),
        true,
    );
    commands.entity(player).insert(LevelEntity);

    let sword = spawn_sword_entity(
        commands,
        sword_visuals,
        Vec3::new(LEVEL_TWO_PLAYER_START_X, GROUND_Y, 4.0),
        SwordState::Equipped,
    );
    commands.entity(sword).insert(LevelEntity);

    spawn_level_two_target_shelf(commands, art);
    spawn_breakable_crate(
        commands,
        art,
        Vec3::new(LEVEL_TWO_CRATE_X, LEVEL_TWO_SHELF_TOP_Y, 5.0),
        CrateReward::CompleteLevelTwo,
    );

    commands.spawn((
        LevelEntity,
        Text2d::new(
            "Left Click to slash.\nHold Right Click to aim the sword.\nRelease to shatter the crate above.",
        ),
        TextFont {
            font: fonts.pixel_regular.clone(),
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgb(0.91, 0.94, 0.98)),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(LEVEL_TWO_HINT_X, LEVEL_TWO_HINT_Y, 4.0),
    ));

    let knight_name = if profile.name.is_empty() {
        "Knight"
    } else {
        profile.name.as_str()
    };

    commands.spawn((
        LevelEntity,
        LevelTwoCompletionText,
        Visibility::Hidden,
        Text2d::new(format!(
            "{knight_name}, the offer letter is yours.\nLevel 3 is deeper in the dungeon."
        )),
        TextFont {
            font: fonts.pixel_regular.clone(),
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.98, 0.92, 0.72)),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, ROOM_CEILING_Y - 30.0, 8.0),
    ));
}

fn spawn_level_two_target_shelf(commands: &mut Commands, art: &LevelArtHandles) {
    let platform_tiles = 2usize;
    let first_tile_x = LEVEL_TWO_CRATE_X - TILE_WORLD_SIZE * 0.5;

    for tile_index in 0..platform_tiles {
        let x = first_tile_x + tile_index as f32 * TILE_WORLD_SIZE;
        spawn_centered_tile(
            commands,
            art.floor_tiles[(tile_index + 1) % art.floor_tiles.len()].clone(),
            Vec3::new(x, LEVEL_TWO_SHELF_TOP_Y - TILE_WORLD_SIZE * 0.5, 1.5),
        );
        spawn_centered_tile(
            commands,
            art.edge_down.clone(),
            Vec3::new(x, LEVEL_TWO_SHELF_TOP_Y - TILE_WORLD_SIZE * 1.5, 1.4),
        );
    }

    for x in [LEVEL_TWO_CRATE_X - 66.0, LEVEL_TWO_CRATE_X + 66.0] {
        spawn_bottom_anchored_sprite(
            commands,
            art.column_wall.clone(),
            Vec3::new(x, LEVEL_TWO_SHELF_TOP_Y - 64.0, 1.6),
            TILE_SCALE,
        );
    }

    spawn_sword_blocker(
        commands,
        Vec2::new(LEVEL_TWO_CRATE_X, LEVEL_TWO_SHELF_TOP_Y - 32.0),
        Vec2::new(80.0, 32.0),
    );
}

fn spawn_breakable_crate(
    commands: &mut Commands,
    art: &LevelArtHandles,
    position: Vec3,
    reward: CrateReward,
) {
    commands.spawn((
        LevelEntity,
        BreakableCrate { reward },
        Sprite::from_image(art.crate_texture.clone()),
        Anchor::BOTTOM_CENTER,
        Transform::from_translation(position).with_scale(Vec3::splat(TILE_SCALE)),
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

fn spawn_tutorial_marker(
    commands: &mut Commands,
    art: &LevelArtHandles,
    fonts: &GameFonts,
    position: Vec3,
) {
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
        TextFont {
            font: fonts.pixel_bold.clone(),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.99, 0.86, 0.34)),
        Transform::from_xyz(position.x, position.y + 120.0, position.z + 1.0),
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

fn spawn_sword_blocker(commands: &mut Commands, center: Vec2, half_extents: Vec2) {
    commands.spawn((
        LevelEntity,
        SwordBlocker { half_extents },
        Transform::from_xyz(center.x, center.y, 2.0),
        GlobalTransform::default(),
    ));
}

pub(crate) fn level_bounds_for(level: LevelId) -> LevelBounds {
    match level {
        LevelId::LevelOne => LevelBounds {
            wall_left_x: ROOM_WALL_LEFT_X,
            wall_right_x: ROOM_WALL_RIGHT_X,
            player_left_x: LEVEL_ONE_PLAYER_START_X,
            player_right_x: ROOM_PLAYER_RIGHT_X,
            ceiling_y: ROOM_CEILING_Y,
        },
        LevelId::LevelTwo => LevelBounds {
            wall_left_x: ROOM_WALL_LEFT_X,
            wall_right_x: ROOM_WALL_RIGHT_X,
            player_left_x: ROOM_PLAYER_LEFT_X,
            player_right_x: ROOM_PLAYER_RIGHT_X,
            ceiling_y: ROOM_CEILING_Y,
        },
    }
}

pub(crate) fn level_camera_focus_x(level: LevelId) -> f32 {
    match level {
        LevelId::LevelOne => LEVEL_ONE_PLAYER_START_X,
        LevelId::LevelTwo => LEVEL_TWO_PLAYER_START_X,
    }
}
