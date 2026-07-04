use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::boss::{
    BOSS_MAX_HP, Boss, BossAnimationFrame, BossAnimationTimer, BossDefeatedText, BossHp,
    BossInvincible, BossPhase, spawn_boss_hp_bar, spawn_player_hp_ui,
};
use crate::fonts::GameFonts;
use crate::level::{
    BreakableChest, BreakableCrate, CrateReward, DQ_TEXT_GOLD, DQ_TEXT_WHITE,
    LEVEL_FIVE_BOSS_START_X,
    LEVEL_FIVE_PLAYER_START_X, LEVEL_FOUR_BLUE_A_X, LEVEL_FOUR_BLUE_B_X, LEVEL_FOUR_DOOR_X,
    LEVEL_FOUR_GREEN_X, LEVEL_FOUR_PLAYER_START_X, LEVEL_FOUR_RED_A_X, LEVEL_FOUR_RED_B_X,
    LEVEL_ONE_CRATE_X, LEVEL_ONE_DOOR_X, LEVEL_ONE_PLAYER_START_X, LEVEL_ONE_SWORD_X,
    LEVEL_ONE_TUTORIAL_X, LEVEL_ONE_WIZARD_X, LEVEL_THREE_BLUE_X, LEVEL_THREE_DOOR_X,
    LEVEL_THREE_GREEN_X, LEVEL_THREE_PLAYER_START_X, LEVEL_THREE_RED_X, LEVEL_TWO_CRATE_X,
    LEVEL_TWO_DOOR_X, LEVEL_TWO_HINT_X, LEVEL_TWO_HINT_Y, LEVEL_TWO_PLAYER_START_X,
    LEVEL_TWO_SHELF_TOP_Y, LevelArtHandles, LevelBounds, LevelEntity, LevelFourCompletionText,
    LevelThreeCompletionText, LevelTwoCompletionText, ROOM_CEILING_Y, ROOM_PLAYER_LEFT_X,
    ROOM_PLAYER_RIGHT_X, ROOM_WALL_LEFT_X, ROOM_WALL_RIGHT_X, RoomTheme, SwordBlocker, TILE_SCALE,
    TILE_WORLD_SIZE, TrainingDoor, WIZARD_SCALE, WizardAnimationFrame, WizardAnimationTimer,
    WizardNpc, frame_level_camera, spawn_bottom_anchored_sprite, spawn_centered_tile,
    spawn_room_shell, spawn_world_text,
};

use crate::player::{GROUND_Y, PlayerAnimationHandles, spawn::spawn_player_entity};
use crate::puzzle::PuzzleBlock;
use crate::state::{BlockColor, PlayerHealth};
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
    mut player_health: ResMut<PlayerHealth>,
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
        &mut player_health,
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
    player_health: &mut PlayerHealth,
) {
    match campaign.current_level {
        LevelId::LevelOne => spawn_level_one(commands, art, fonts, player_anims, sword_visuals),
        LevelId::LevelTwo => {
            spawn_level_two(commands, art, fonts, player_anims, sword_visuals, profile)
        }
        LevelId::LevelThree => spawn_level_three(
            commands,
            art,
            fonts,
            player_anims,
            sword_visuals,
            campaign,
            profile,
        ),
        LevelId::LevelFour => spawn_level_four(
            commands,
            art,
            fonts,
            player_anims,
            sword_visuals,
            campaign,
            profile,
        ),
        LevelId::LevelFive => spawn_level_five(
            commands,
            art,
            fonts,
            player_anims,
            sword_visuals,
            profile,
            player_health,
        ),
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

    spawn_room_shell(commands, art, fonts, "LEVEL 1", RoomTheme::Castle);

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

    spawn_world_text(
        commands,
        fonts.pixel_regular.clone(),
        "[ E ] Pick up",
        18.0,
        DQ_TEXT_WHITE,
        Vec3::new(LEVEL_ONE_SWORD_X, GROUND_Y + 96.0, 5.0),
    );

    spawn_wizard(commands, art, Vec3::new(LEVEL_ONE_WIZARD_X, GROUND_Y, 4.0));

    spawn_bottom_anchored_sprite(
        commands,
        art.flask_big_blue.clone(),
        Vec3::new(LEVEL_ONE_WIZARD_X + 52.0, GROUND_Y, 4.0),
        TILE_SCALE,
    );
    spawn_bottom_anchored_sprite(
        commands,
        art.flask_big_red.clone(),
        Vec3::new(LEVEL_ONE_WIZARD_X + 84.0, GROUND_Y, 4.0),
        TILE_SCALE,
    );

    spawn_chest(
        commands,
        art,
        Vec3::new(LEVEL_ONE_TUTORIAL_X - 80.0, GROUND_Y, 4.0),
    );

    spawn_bottom_anchored_sprite(
        commands,
        art.skull.clone(),
        Vec3::new(-560.0, GROUND_Y, 4.0),
        TILE_SCALE,
    );

    spawn_breakable_crate(
        commands,
        art,
        Vec3::new(LEVEL_ONE_CRATE_X, GROUND_Y, 4.0),
        CrateReward::OpenTrainingDoor,
    );
    spawn_world_text(
        commands,
        fonts.pixel_regular.clone(),
        "Left Click to slash the crate.",
        16.0,
        DQ_TEXT_WHITE,
        Vec3::new(LEVEL_ONE_CRATE_X, GROUND_Y + 112.0, 4.0),
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

    spawn_room_shell(commands, art, fonts, "LEVEL 2", RoomTheme::Castle);

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

    spawn_bottom_anchored_sprite(
        commands,
        art.skull.clone(),
        Vec3::new(-350.0, GROUND_Y, 4.0),
        TILE_SCALE,
    );
    spawn_chest(commands, art, Vec3::new(330.0, GROUND_Y, 4.0));

    spawn_training_door(
        commands,
        art,
        Vec3::new(LEVEL_TWO_DOOR_X, GROUND_Y, 4.0),
        false,
    );

    spawn_breakable_crate(
        commands,
        art,
        Vec3::new(LEVEL_TWO_CRATE_X, LEVEL_TWO_SHELF_TOP_Y, 5.0),
        CrateReward::CompleteLevelTwo,
    );

    spawn_world_text(
        commands,
        fonts.pixel_regular.clone(),
        "Hold Right Click to aim the sword.\nRelease to shatter the crate above.",
        16.0,
        DQ_TEXT_WHITE,
        Vec3::new(LEVEL_TWO_HINT_X, LEVEL_TWO_HINT_Y, 4.0),
    );

    let knight_name = if profile.name.is_empty() {
        "Knight"
    } else {
        profile.name.as_str()
    };

    let completion = spawn_world_text(
        commands,
        fonts.pixel_regular.clone(),
        &format!("{knight_name}, the offer letter is yours.\nLevel 3 is deeper in the dungeon."),
        19.0,
        DQ_TEXT_GOLD,
        Vec3::new(0.0, ROOM_CEILING_Y - 40.0, 8.0),
    );
    commands
        .entity(completion)
        .insert((LevelTwoCompletionText, Visibility::Hidden));
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

fn spawn_chest(commands: &mut Commands, art: &LevelArtHandles, position: Vec3) {
    commands.spawn((
        LevelEntity,
        BreakableChest { opened: false },
        Sprite::from_image(art.chest_closed.clone()),
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
        Transform::from_translation(position).with_scale(Vec3::splat(WIZARD_SCALE)),
    ));
}

fn spawn_training_door(commands: &mut Commands, art: &LevelArtHandles, position: Vec3, open: bool) {
    // The leaf tile carries its own stone arch; the tileset's separate frame
    // pieces assume matching light bricks and look broken on our dark walls.
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
}

/// Spawns a puzzle block as a gem medallion: a rotated diamond gem set in a
/// gold-trimmed stone frame. The `PuzzleBlock` entity is the gem sprite itself,
/// so the existing hit/recolor logic keeps working untouched.
fn spawn_puzzle_gem(
    commands: &mut Commands,
    fonts: &GameFonts,
    color: BlockColor,
    position: Vec3,
) {
    let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4);

    commands
        .spawn((
            LevelEntity,
            PuzzleBlock {
                color,
                activated: false,
                hit_cooldown: 0.0,
            },
            Sprite::from_color(color.dim_color(), Vec2::splat(34.0)),
            Transform::from_translation(position).with_rotation(rotation),
        ))
        .with_children(|gem| {
            gem.spawn((
                Sprite::from_color(Color::srgb(0.07, 0.06, 0.10), Vec2::splat(54.0)),
                Transform::from_xyz(0.0, 0.0, -0.03),
            ));
            gem.spawn((
                Sprite::from_color(Color::srgb(0.62, 0.52, 0.30), Vec2::splat(46.0)),
                Transform::from_xyz(0.0, 0.0, -0.02),
            ));
            gem.spawn((
                Sprite::from_color(Color::srgb(0.05, 0.04, 0.08), Vec2::splat(40.0)),
                Transform::from_xyz(0.0, 0.0, -0.01),
            ));
            // specular glint on the upper facet
            gem.spawn((
                Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.32), Vec2::splat(7.0)),
                Transform::from_xyz(-7.0, 7.0, 0.01),
            ));
        });

    commands.spawn((
        LevelEntity,
        Text2d::new(color.label()),
        TextFont {
            font: fonts.pixel_bold.clone(),
            font_size: 15.0,
            ..default()
        },
        TextColor(color.bright_color()),
        Transform::from_xyz(position.x, position.y - 54.0, position.z),
    ));
}

/// Sequence display: the ordered gems as mini medallions (same gold-trimmed
/// diamond style as the real puzzle gems), floating bare over the wall with
/// the instruction line below — no window, per the game's clean-text look.
fn spawn_sequence_display(
    commands: &mut Commands,
    fonts: &GameFonts,
    sequence: &[BlockColor],
    center_x: f32,
    instruction: &str,
) {
    let y = ROOM_CEILING_Y + 6.0;
    let spacing = 78.0;

    let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4);
    let start_x = center_x - (sequence.len() as f32 - 1.0) * spacing * 0.5;
    for (i, color) in sequence.iter().enumerate() {
        let x = start_x + i as f32 * spacing;
        commands
            .spawn((
                LevelEntity,
                Sprite::from_color(color.bright_color(), Vec2::splat(30.0)),
                Transform::from_xyz(x, y + 30.0, 5.0).with_rotation(rotation),
            ))
            .with_children(|gem| {
                gem.spawn((
                    Sprite::from_color(Color::srgb(0.07, 0.06, 0.10), Vec2::splat(48.0)),
                    Transform::from_xyz(0.0, 0.0, -0.03),
                ));
                gem.spawn((
                    Sprite::from_color(Color::srgb(0.62, 0.52, 0.30), Vec2::splat(41.0)),
                    Transform::from_xyz(0.0, 0.0, -0.02),
                ));
                gem.spawn((
                    Sprite::from_color(Color::srgb(0.05, 0.04, 0.08), Vec2::splat(35.0)),
                    Transform::from_xyz(0.0, 0.0, -0.01),
                ));
                gem.spawn((
                    Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.32), Vec2::splat(6.0)),
                    Transform::from_xyz(-6.0, 6.0, 0.01),
                ));
            });
        spawn_world_text(
            commands,
            fonts.pixel_bold.clone(),
            &format!("{}", i + 1),
            15.0,
            DQ_TEXT_GOLD,
            Vec3::new(x, y - 16.0, 5.0),
        );
    }

    spawn_world_text(
        commands,
        fonts.pixel_regular.clone(),
        instruction,
        15.0,
        DQ_TEXT_WHITE,
        Vec3::new(center_x, y - 46.0, 5.0),
    );
}

fn spawn_level_three(
    commands: &mut Commands,
    art: &LevelArtHandles,
    fonts: &GameFonts,
    player_anims: &PlayerAnimationHandles,
    sword_visuals: &SwordVisualHandles,
    campaign: &CampaignState,
    profile: &PlayerProfile,
) {
    commands.insert_resource(level_bounds_for(LevelId::LevelThree));

    spawn_room_shell(commands, art, fonts, "LEVEL 3", RoomTheme::Castle);

    let player = spawn_player_entity(
        commands,
        player_anims,
        Vec3::new(LEVEL_THREE_PLAYER_START_X, GROUND_Y, 5.0),
        true,
    );
    commands.entity(player).insert(LevelEntity);

    let sword = spawn_sword_entity(
        commands,
        sword_visuals,
        Vec3::new(LEVEL_THREE_PLAYER_START_X, GROUND_Y, 4.0),
        SwordState::Equipped,
    );
    commands.entity(sword).insert(LevelEntity);

    let blocks = [
        (LEVEL_THREE_GREEN_X, BlockColor::Green),
        (LEVEL_THREE_RED_X, BlockColor::Red),
        (LEVEL_THREE_BLUE_X, BlockColor::Blue),
    ];
    for (x, color) in blocks {
        spawn_puzzle_gem(commands, fonts, color, Vec3::new(x, GROUND_Y + 52.0, 4.0));
    }

    spawn_sequence_display(
        commands,
        fonts,
        &campaign.puzzle_sequence,
        0.0,
        "Strike the gems in the order shown.",
    );

    spawn_training_door(
        commands,
        art,
        Vec3::new(LEVEL_THREE_DOOR_X, GROUND_Y, 4.0),
        false,
    );

    let knight_name = if profile.name.is_empty() {
        "Knight"
    } else {
        profile.name.as_str()
    };
    let completion = spawn_world_text(
        commands,
        fonts.pixel_regular.clone(),
        &format!("{knight_name}, you have mastered the sequence.\nThe dungeon is yours."),
        19.0,
        DQ_TEXT_GOLD,
        Vec3::new(0.0, GROUND_Y + 150.0, 8.0),
    );
    commands
        .entity(completion)
        .insert((LevelThreeCompletionText, Visibility::Hidden));
}

fn spawn_level_four(
    commands: &mut Commands,
    art: &LevelArtHandles,
    fonts: &GameFonts,
    player_anims: &PlayerAnimationHandles,
    sword_visuals: &SwordVisualHandles,
    campaign: &CampaignState,
    profile: &PlayerProfile,
) {
    commands.insert_resource(level_bounds_for(LevelId::LevelFour));

    spawn_room_shell(commands, art, fonts, "LEVEL 4", RoomTheme::Castle);

    let player = spawn_player_entity(
        commands,
        player_anims,
        Vec3::new(LEVEL_FOUR_PLAYER_START_X, GROUND_Y, 5.0),
        true,
    );
    commands.entity(player).insert(LevelEntity);

    let sword = spawn_sword_entity(
        commands,
        sword_visuals,
        Vec3::new(LEVEL_FOUR_PLAYER_START_X, GROUND_Y, 4.0),
        SwordState::Equipped,
    );
    commands.entity(sword).insert(LevelEntity);

    // All 5 blocks are elevated — every step requires a throw.
    // Blue_A/B are spawned first so the puzzle system hits Blue_A before Blue_B.
    for (x, color) in [
        (LEVEL_FOUR_BLUE_A_X, BlockColor::Blue),
        (LEVEL_FOUR_RED_A_X, BlockColor::Red),
        (LEVEL_FOUR_GREEN_X, BlockColor::Green),
        (LEVEL_FOUR_RED_B_X, BlockColor::Red),
        (LEVEL_FOUR_BLUE_B_X, BlockColor::Blue),
    ] {
        spawn_bottom_anchored_sprite(
            commands,
            art.column_wall.clone(),
            Vec3::new(x, GROUND_Y, 2.0),
            TILE_SCALE,
        );
        spawn_puzzle_gem(commands, fonts, color, Vec3::new(x, GROUND_Y + 192.0, 4.0));
    }

    spawn_sequence_display(
        commands,
        fonts,
        &campaign.puzzle_sequence,
        0.0,
        "Throw the sword to strike the gems in order.",
    );

    spawn_training_door(
        commands,
        art,
        Vec3::new(LEVEL_FOUR_DOOR_X, GROUND_Y, 4.0),
        false,
    );

    let knight_name = if profile.name.is_empty() {
        "Knight"
    } else {
        profile.name.as_str()
    };
    let completion = spawn_world_text(
        commands,
        fonts.pixel_regular.clone(),
        &format!("{knight_name}, The Vault has fallen.\nYou are a true master of the sword."),
        19.0,
        DQ_TEXT_GOLD,
        Vec3::new(0.0, GROUND_Y + 150.0, 8.0),
    );
    commands
        .entity(completion)
        .insert((LevelFourCompletionText, Visibility::Hidden));
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
    let standard = LevelBounds {
        wall_left_x: ROOM_WALL_LEFT_X,
        wall_right_x: ROOM_WALL_RIGHT_X,
        player_left_x: ROOM_PLAYER_LEFT_X,
        player_right_x: ROOM_PLAYER_RIGHT_X,
        ceiling_y: ROOM_CEILING_Y,
    };
    match level {
        LevelId::LevelOne => LevelBounds {
            player_left_x: LEVEL_ONE_PLAYER_START_X,
            ..standard
        },
        LevelId::LevelTwo | LevelId::LevelThree | LevelId::LevelFour | LevelId::LevelFive => {
            standard
        }
    }
}

pub(crate) fn level_camera_focus_x(level: LevelId) -> f32 {
    match level {
        LevelId::LevelOne => LEVEL_ONE_PLAYER_START_X,
        LevelId::LevelTwo => LEVEL_TWO_PLAYER_START_X,
        LevelId::LevelThree => LEVEL_THREE_PLAYER_START_X,
        LevelId::LevelFour => LEVEL_FOUR_PLAYER_START_X,
        LevelId::LevelFive => LEVEL_FIVE_PLAYER_START_X,
    }
}

fn spawn_level_five(
    commands: &mut Commands,
    art: &LevelArtHandles,
    fonts: &GameFonts,
    player_anims: &PlayerAnimationHandles,
    sword_visuals: &SwordVisualHandles,
    profile: &PlayerProfile,
    players_health: &mut PlayerHealth,
) {
    *players_health = PlayerHealth::default();

    commands.insert_resource(level_bounds_for(LevelId::LevelFive));

    spawn_room_shell(commands, art, fonts, "THE WIZARD'S LAIR", RoomTheme::Lair);

    // Blood-red atmosphere overlay
    commands.spawn((
        LevelEntity,
        Sprite::from_color(
            Color::srgba(0.35, 0.0, 0.0, 0.18),
            Vec2::new(2800.0, 1800.0),
        ),
        Transform::from_xyz(0.0, 0.0, -37.0),
    ));
    // Dark crimson ground fog
    commands.spawn((
        LevelEntity,
        Sprite::from_color(Color::srgba(0.5, 0.0, 0.0, 0.12), Vec2::new(2800.0, 180.0)),
        Transform::from_xyz(0.0, GROUND_Y + 90.0, -20.0),
    ));

    let player = spawn_player_entity(
        commands,
        player_anims,
        Vec3::new(LEVEL_FIVE_PLAYER_START_X, GROUND_Y, 5.0),
        true,
    );
    commands.entity(player).insert(LevelEntity);

    let sword = spawn_sword_entity(
        commands,
        sword_visuals,
        Vec3::new(LEVEL_FIVE_PLAYER_START_X, GROUND_Y, 4.0),
        SwordState::Equipped,
    );
    commands.entity(sword).insert(LevelEntity);

    commands.spawn((
        LevelEntity,
        Boss,
        BossHp {
            current: BOSS_MAX_HP,
            max: BOSS_MAX_HP,
        },
        BossPhase::Chase,
        BossInvincible(0.0),
        BossAnimationFrame::default(),
        BossAnimationTimer(Timer::from_seconds(0.14, TimerMode::Repeating)),
        Sprite {
            image: art.wizard_idle_frames[0].clone(),
            color: Color::srgb(1.0, 0.28, 0.28),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
        Transform::from_xyz(LEVEL_FIVE_BOSS_START_X, GROUND_Y + 80.0, 4.0)
            .with_scale(Vec3::splat(WIZARD_SCALE)),
    ));

    spawn_boss_hp_bar(commands, fonts);
    spawn_player_hp_ui(commands, fonts, art);

    // Menacing dressing – dense skull placement and extra columns.
    // (The Lair room theme already hangs red banners and lava fountains.)
    for x in [-680.0, -540.0, -380.0, -180.0, 80.0, 300.0, 500.0, 680.0] {
        spawn_bottom_anchored_sprite(
            commands,
            art.skull.clone(),
            Vec3::new(x, GROUND_Y, 4.0),
            TILE_SCALE,
        );
    }

    // Extra columns flanking the arena
    for x in [-200.0, 200.0] {
        spawn_bottom_anchored_sprite(
            commands,
            art.column_wall.clone(),
            Vec3::new(x, GROUND_Y, 2.0),
            TILE_SCALE,
        );
    }

    // Wall holes – more damage than normal rooms
    for (x, texture) in [
        (-500.0, art.wall_hole_1.clone()),
        (-300.0, art.wall_hole_2.clone()),
        (300.0, art.wall_hole_1.clone()),
        (500.0, art.wall_hole_2.clone()),
    ] {
        spawn_centered_tile(
            commands,
            texture,
            Vec3::new(x, GROUND_Y + TILE_WORLD_SIZE * 2.5, -8.0),
        );
    }

    spawn_world_text(
        commands,
        fonts.pixel_bold.clone(),
        "Only the thrown blade can pierce his dark magic.",
        18.0,
        Color::srgb(0.96, 0.55, 0.50),
        Vec3::new(0.0, ROOM_CEILING_Y + 2.0, 5.0),
    );

    let knight_name = if profile.name.is_empty() {
        "Knight"
    } else {
        profile.name.as_str()
    };
    let defeated_text = spawn_world_text(
        commands,
        fonts.pixel_regular.clone(),
        &format!("The Dark Wizard falls.\n{knight_name}, your internship is confirmed."),
        20.0,
        Color::srgb(1.0, 0.85, 0.3),
        Vec3::new(0.0, GROUND_Y + 160.0, 8.0),
    );
    commands
        .entity(defeated_text)
        .insert((BossDefeatedText, Visibility::Hidden));
}
