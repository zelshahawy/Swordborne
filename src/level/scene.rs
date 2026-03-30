use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::PrimaryWindow;

use crate::fonts::GameFonts;
use crate::level::{
    LevelArtHandles, LevelBounds, LevelEntity, ROOM_CEILING_Y, ROOM_TILE_COLUMNS, ROOM_WALL_LEFT_X,
    ROOM_WALL_RIGHT_X, ROOM_WALL_ROWS, TILE_SCALE, TILE_WORLD_SIZE,
};

use crate::player::{GROUND_Y, Player};

const LEVEL_CAMERA_Y: f32 = 90.0;
const LEVEL_CAMERA_SCALE: f32 = 1.2;
const LEVEL_CAMERA_SMOOTHING: f32 = 8.0;
const LEVEL_LABEL_Y: f32 = ROOM_CEILING_Y + 8.0;
const BACKGROUND_SIDE_PADDING: isize = 10;
const BACKGROUND_ROWS_BELOW_GROUND: usize = 12;
const BACKGROUND_ROWS_ABOVE_CEILING: usize = 3;

pub(crate) fn frame_level_camera(
    camera_query: &mut Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    window_query: Option<&Query<&Window, With<PrimaryWindow>>>,
    bounds: Option<LevelBounds>,
    player_x: Option<f32>,
) {
    let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let Projection::Orthographic(orthographic) = &mut *projection else {
        return;
    };
    orthographic.scale = LEVEL_CAMERA_SCALE;
    camera_transform.translation.y = LEVEL_CAMERA_Y;

    camera_transform.translation.x = match (window_query, bounds, player_x) {
        (Some(window_query), Some(bounds), Some(player_x)) => match window_query.single() {
            Ok(window) => camera_target_x(window, orthographic.scale, bounds, player_x),
            Err(_) => 0.0,
        },
        _ => 0.0,
    };
}

pub(crate) fn update_level_camera(
    time: Res<Time>,
    bounds: Res<LevelBounds>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<(&mut Transform, &mut Projection), (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let Projection::Orthographic(orthographic) = &mut *projection else {
        return;
    };
    orthographic.scale = LEVEL_CAMERA_SCALE;

    let target_x = camera_target_x(
        window,
        orthographic.scale,
        *bounds,
        player_transform.translation.x,
    );
    let lerp_factor = 1.0 - (-LEVEL_CAMERA_SMOOTHING * time.delta_secs()).exp();

    camera_transform.translation.x += (target_x - camera_transform.translation.x) * lerp_factor;
    camera_transform.translation.y = LEVEL_CAMERA_Y;
}

pub(crate) fn spawn_room_shell(
    commands: &mut Commands,
    art: &LevelArtHandles,
    fonts: &GameFonts,
    level_label: &str,
) {
    commands.spawn((
        LevelEntity,
        Sprite::from_color(Color::srgb(0.02, 0.025, 0.04), Vec2::new(2800.0, 1800.0)),
        Transform::from_xyz(0.0, 0.0, -38.0),
    ));

    commands.spawn((
        LevelEntity,
        Sprite::from_color(
            Color::srgba(0.04, 0.05, 0.08, 0.86),
            Vec2::new(2800.0, 700.0),
        ),
        Transform::from_xyz(0.0, ROOM_CEILING_Y + 210.0, -30.0),
    ));

    commands.spawn((
        LevelEntity,
        Sprite::from_color(
            Color::srgba(0.08, 0.1, 0.14, 0.22),
            Vec2::new(2800.0, 1200.0),
        ),
        Transform::from_xyz(0.0, GROUND_Y + 120.0, -26.0),
    ));

    // Sky — anchored at ROOM_CEILING_Y, z=-13 so background brick tiles (z=-12) render in
    // front of it. The BACKGROUND_ROWS_ABOVE_CEILING brick rows overlap the sky naturally,
    // eliminating any seam. Height 4000 covers from ceiling to well past screen top.
    commands.spawn((
        LevelEntity,
        Sprite {
            image: art.sky.clone(),
            custom_size: Some(Vec2::new(4000.0, 4000.0)),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.0,
            },
            ..default()
        },
        Transform::from_xyz(0.0, ROOM_CEILING_Y + 2000.0, -13.0),
    ));

    spawn_background_tiles(commands, art);
    spawn_floor_strip(commands, art);
    spawn_side_walls(commands, art);
    spawn_decor(commands, art);
    spawn_underground_decor(commands, art);
    spawn_level_label(commands, fonts, level_label);
}

fn spawn_background_tiles(commands: &mut Commands, art: &LevelArtHandles) {
    let start_column = -BACKGROUND_SIDE_PADDING;
    let end_column = ROOM_TILE_COLUMNS as isize + BACKGROUND_SIDE_PADDING;
    let total_rows = ROOM_WALL_ROWS + BACKGROUND_ROWS_BELOW_GROUND + BACKGROUND_ROWS_ABOVE_CEILING;
    let bottom_y = GROUND_Y - TILE_WORLD_SIZE * (BACKGROUND_ROWS_BELOW_GROUND as f32 - 0.5);

    for row in 0..total_rows {
        let y = bottom_y + row as f32 * TILE_WORLD_SIZE;

        for column in start_column..end_column {
            let x = room_floor_x_signed(column);
            let texture = if row == total_rows - 1 {
                if column == start_column {
                    art.wall_top_left.clone()
                } else if column == end_column - 1 {
                    art.wall_top_right.clone()
                } else {
                    art.wall_top_mid.clone()
                }
            } else {
                art.wall_mid.clone()
            };

            spawn_centered_tile(commands, texture, Vec3::new(x, y, -12.0));
        }
    }
}

fn spawn_floor_strip(commands: &mut Commands, art: &LevelArtHandles) {
    let start_column = -BACKGROUND_SIDE_PADDING;
    let end_column = ROOM_TILE_COLUMNS as isize + BACKGROUND_SIDE_PADDING;

    for column in start_column..end_column {
        let x = room_floor_x_signed(column);
        let texture = art.floor_tiles
            [(column.unsigned_abs() * 3 + column.unsigned_abs() / 2) % art.floor_tiles.len()]
        .clone();

        spawn_centered_tile(
            commands,
            texture,
            Vec3::new(x, GROUND_Y - TILE_WORLD_SIZE * 0.5, 0.0),
        );

        spawn_centered_tile(
            commands,
            art.wall_mid.clone(),
            Vec3::new(x, GROUND_Y - TILE_WORLD_SIZE * 1.5, -1.0),
        );
        spawn_centered_tile(
            commands,
            art.wall_mid.clone(),
            Vec3::new(x, GROUND_Y - TILE_WORLD_SIZE * 2.5, -1.0),
        );
    }
}

fn spawn_side_walls(commands: &mut Commands, art: &LevelArtHandles) {
    let top_y = ROOM_CEILING_Y + TILE_WORLD_SIZE * 0.5;

    spawn_centered_tile(
        commands,
        art.wall_outer_top_left.clone(),
        Vec3::new(ROOM_WALL_LEFT_X, top_y, 0.0),
    );
    spawn_centered_tile(
        commands,
        art.wall_outer_top_right.clone(),
        Vec3::new(ROOM_WALL_RIGHT_X, top_y, 0.0),
    );

    for row in 0..ROOM_WALL_ROWS {
        let y = GROUND_Y + TILE_WORLD_SIZE * 0.5 + row as f32 * TILE_WORLD_SIZE;
        let left_texture = if row == 0 {
            art.wall_outer_front_left.clone()
        } else {
            art.wall_outer_mid_left.clone()
        };
        let right_texture = if row == 0 {
            art.wall_outer_front_right.clone()
        } else {
            art.wall_outer_mid_right.clone()
        };

        spawn_centered_tile(commands, left_texture, Vec3::new(ROOM_WALL_LEFT_X, y, 0.0));
        spawn_centered_tile(
            commands,
            right_texture,
            Vec3::new(ROOM_WALL_RIGHT_X, y, 0.0),
        );
    }
}

fn spawn_decor(commands: &mut Commands, art: &LevelArtHandles) {
    for (x, texture) in [
        (-620.0, art.banner_green.clone()),
        (-250.0, art.banner_blue.clone()),
        (250.0, art.banner_red.clone()),
        (620.0, art.banner_yellow.clone()),
    ] {
        spawn_bottom_anchored_sprite(
            commands,
            texture,
            Vec3::new(x, ROOM_CEILING_Y + 220.0, 1.0),
            TILE_SCALE,
        );
    }

    for x in [-470.0, 470.0] {
        spawn_fountain(commands, art, x);
    }

    for x in [-636.0, -360.0, 360.0, 636.0] {
        spawn_bottom_anchored_sprite(
            commands,
            art.column_wall.clone(),
            Vec3::new(x, GROUND_Y, 1.0),
            TILE_SCALE,
        );
    }

    // Wall holes for dungeon atmosphere — snapped to tile grid, mid-wall height
    for (x, texture) in [
        (-150.0, art.wall_hole_1.clone()),
        (150.0, art.wall_hole_2.clone()),
    ] {
        spawn_centered_tile(
            commands,
            texture,
            Vec3::new(x, GROUND_Y + TILE_WORLD_SIZE * 2.5, -8.0),
        );
    }
}

fn spawn_underground_decor(commands: &mut Commands, art: &LevelArtHandles) {
    // Skulls scattered at varying depths — give the underground a burial feel
    for (x, depth) in [
        (-560.0, 1.5_f32),
        (-320.0, 3.0),
        (-80.0, 2.0),
        (120.0, 4.0),
        (350.0, 2.5),
        (560.0, 1.5),
        (-200.0, 5.0),
        (200.0, 5.5),
    ] {
        spawn_centered_tile(
            commands,
            art.skull.clone(),
            Vec3::new(x, GROUND_Y - TILE_WORLD_SIZE * depth, 1.0),
        );
    }

    // Wall holes embedded in the underground walls — suggest hidden passages
    for (x, depth, tex) in [
        (-100.0, 4.5_f32, art.wall_hole_1.clone()),
        (340.0, 3.0, art.wall_hole_2.clone()),
    ] {
        spawn_centered_tile(
            commands,
            tex,
            Vec3::new(x, GROUND_Y - TILE_WORLD_SIZE * depth, -1.0),
        );
    }

    // edge_down tiles form a ragged cave-bottom border at ~7 rows below ground
    let edge_y = GROUND_Y - TILE_WORLD_SIZE * 7.0;
    let start_col = -BACKGROUND_SIDE_PADDING;
    let end_col = ROOM_TILE_COLUMNS as isize + BACKGROUND_SIDE_PADDING;
    for col in start_col..end_col {
        let x = room_floor_x_signed(col);
        spawn_centered_tile(commands, art.edge_down.clone(), Vec3::new(x, edge_y, -1.0));
    }
}

fn spawn_level_label(commands: &mut Commands, fonts: &GameFonts, level_label: &str) {
    commands.spawn((
        LevelEntity,
        Sprite::from_color(Color::srgba(0.02, 0.03, 0.05, 0.84), Vec2::new(210.0, 50.0)),
        Transform::from_xyz(0.0, LEVEL_LABEL_Y, 3.0),
    ));

    commands.spawn((
        LevelEntity,
        Text2d::new(level_label.to_string()),
        TextFont {
            font: fonts.pixel_bold.clone(),
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.92, 0.97)),
        Transform::from_xyz(0.0, LEVEL_LABEL_Y, 4.0),
    ));
}

fn spawn_fountain(commands: &mut Commands, art: &LevelArtHandles, x: f32) {
    spawn_centered_tile(
        commands,
        art.fountain_top_frames[0].clone(),
        Vec3::new(x, GROUND_Y + 240.0, 0.5),
    );
    spawn_centered_tile(
        commands,
        art.fountain_mid_blue_frames[0].clone(),
        Vec3::new(x, GROUND_Y + 176.0, 0.5),
    );
    spawn_centered_tile(
        commands,
        art.fountain_blue_frames[0].clone(),
        Vec3::new(x, GROUND_Y + 112.0, 0.6),
    );
}

pub(crate) fn spawn_centered_tile(commands: &mut Commands, texture: Handle<Image>, position: Vec3) {
    commands.spawn((
        LevelEntity,
        Sprite::from_image(texture),
        Transform::from_translation(position).with_scale(Vec3::splat(TILE_SCALE)),
    ));
}

pub(crate) fn spawn_bottom_anchored_sprite(
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

fn room_floor_x_signed(index: isize) -> f32 {
    ROOM_WALL_LEFT_X + TILE_WORLD_SIZE * 0.5 + index as f32 * TILE_WORLD_SIZE
}

fn camera_target_x(window: &Window, scale: f32, bounds: LevelBounds, player_x: f32) -> f32 {
    let half_view_width = window.width() * scale * 0.5;
    let min_x = bounds.wall_left_x + half_view_width;
    let max_x = bounds.wall_right_x - half_view_width;

    if min_x >= max_x {
        0.0
    } else {
        player_x.clamp(min_x, max_x)
    }
}
