use bevy::prelude::*;

const WIZARD_IDLE_FRAMES: usize = 4;
const FLOOR_VARIANTS: usize = 8;
const FOUNTAIN_FRAMES: usize = 3;
const CHEST_FRAMES: usize = 3;

#[derive(Resource)]
pub(super) struct LevelArtHandles {
    pub floor_tiles: [Handle<Image>; FLOOR_VARIANTS],
    pub wall_mid: Handle<Image>,
    pub wall_outer_top_left: Handle<Image>,
    pub wall_outer_top_right: Handle<Image>,
    pub wall_outer_mid_left: Handle<Image>,
    pub wall_outer_mid_right: Handle<Image>,
    pub wall_outer_front_left: Handle<Image>,
    pub wall_outer_front_right: Handle<Image>,
    pub wall_top_left: Handle<Image>,
    pub wall_top_mid: Handle<Image>,
    pub wall_top_right: Handle<Image>,
    pub column_wall: Handle<Image>,
    pub banner_blue: Handle<Image>,
    pub banner_green: Handle<Image>,
    pub banner_red: Handle<Image>,
    pub banner_yellow: Handle<Image>,
    pub fountain_top_frames: [Handle<Image>; FOUNTAIN_FRAMES],
    pub fountain_blue_frames: [Handle<Image>; FOUNTAIN_FRAMES],
    pub edge_down: Handle<Image>,
    pub crate_texture: Handle<Image>,
    pub door_closed: Handle<Image>,
    pub door_open: Handle<Image>,
    pub tutorial_base: Handle<Image>,
    pub chest_frames: [Handle<Image>; CHEST_FRAMES],
    pub wizard_idle_frames: [Handle<Image>; WIZARD_IDLE_FRAMES],
}

pub(super) fn load_level_art(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LevelArtHandles {
        floor_tiles: [
            asset_server.load("dungeon/frames/floor_1.png"),
            asset_server.load("dungeon/frames/floor_2.png"),
            asset_server.load("dungeon/frames/floor_3.png"),
            asset_server.load("dungeon/frames/floor_4.png"),
            asset_server.load("dungeon/frames/floor_5.png"),
            asset_server.load("dungeon/frames/floor_6.png"),
            asset_server.load("dungeon/frames/floor_7.png"),
            asset_server.load("dungeon/frames/floor_8.png"),
        ],
        wall_mid: asset_server.load("dungeon/frames/wall_mid.png"),
        wall_outer_top_left: asset_server.load("dungeon/frames/wall_outer_top_left.png"),
        wall_outer_top_right: asset_server.load("dungeon/frames/wall_outer_top_right.png"),
        wall_outer_mid_left: asset_server.load("dungeon/frames/wall_outer_mid_left.png"),
        wall_outer_mid_right: asset_server.load("dungeon/frames/wall_outer_mid_right.png"),
        wall_outer_front_left: asset_server.load("dungeon/frames/wall_outer_front_left.png"),
        wall_outer_front_right: asset_server.load("dungeon/frames/wall_outer_front_right.png"),
        wall_top_left: asset_server.load("dungeon/frames/wall_top_left.png"),
        wall_top_mid: asset_server.load("dungeon/frames/wall_top_mid.png"),
        wall_top_right: asset_server.load("dungeon/frames/wall_top_right.png"),
        column_wall: asset_server.load("dungeon/frames/column_wall.png"),
        banner_blue: asset_server.load("dungeon/frames/wall_banner_blue.png"),
        banner_green: asset_server.load("dungeon/frames/wall_banner_green.png"),
        banner_red: asset_server.load("dungeon/frames/wall_banner_red.png"),
        banner_yellow: asset_server.load("dungeon/frames/wall_banner_yellow.png"),
        fountain_top_frames: [
            asset_server.load("dungeon/frames/wall_fountain_top_1.png"),
            asset_server.load("dungeon/frames/wall_fountain_top_2.png"),
            asset_server.load("dungeon/frames/wall_fountain_top_3.png"),
        ],
        fountain_blue_frames: [
            asset_server.load("dungeon/frames/wall_fountain_basin_blue_anim_f0.png"),
            asset_server.load("dungeon/frames/wall_fountain_basin_blue_anim_f1.png"),
            asset_server.load("dungeon/frames/wall_fountain_basin_blue_anim_f2.png"),
        ],
        edge_down: asset_server.load("dungeon/frames/edge_down.png"),
        crate_texture: asset_server.load("dungeon/frames/crate.png"),
        door_closed: asset_server.load("dungeon/frames/doors_leaf_closed.png"),
        door_open: asset_server.load("dungeon/frames/doors_leaf_open.png"),
        tutorial_base: asset_server.load("dungeon/frames/column.png"),
        chest_frames: [
            asset_server.load("dungeon/frames/chest_full_open_anim_f0.png"),
            asset_server.load("dungeon/frames/chest_full_open_anim_f1.png"),
            asset_server.load("dungeon/frames/chest_full_open_anim_f2.png"),
        ],
        wizard_idle_frames: [
            asset_server.load("dungeon/frames/wizzard_m_idle_anim_f0.png"),
            asset_server.load("dungeon/frames/wizzard_m_idle_anim_f1.png"),
            asset_server.load("dungeon/frames/wizzard_m_idle_anim_f2.png"),
            asset_server.load("dungeon/frames/wizzard_m_idle_anim_f3.png"),
        ],
    });
}
