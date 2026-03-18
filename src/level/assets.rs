use bevy::prelude::*;

const WIZARD_IDLE_FRAMES: usize = 4;

#[derive(Resource)]
pub(super) struct LevelArtHandles {
    pub floor: Handle<Image>,
    pub wall_mid: Handle<Image>,
    pub wall_top_left: Handle<Image>,
    pub wall_top_mid: Handle<Image>,
    pub wall_top_right: Handle<Image>,
    pub column_wall: Handle<Image>,
    pub banner_blue: Handle<Image>,
    pub banner_red: Handle<Image>,
    pub crate_texture: Handle<Image>,
    pub door_closed: Handle<Image>,
    pub door_open: Handle<Image>,
    pub tutorial_base: Handle<Image>,
    pub wizard_idle_frames: [Handle<Image>; WIZARD_IDLE_FRAMES],
}

pub(super) fn load_level_art(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LevelArtHandles {
        floor: asset_server.load("dungeon/frames/floor_2.png"),
        wall_mid: asset_server.load("dungeon/frames/wall_mid.png"),
        wall_top_left: asset_server.load("dungeon/frames/wall_top_left.png"),
        wall_top_mid: asset_server.load("dungeon/frames/wall_top_mid.png"),
        wall_top_right: asset_server.load("dungeon/frames/wall_top_right.png"),
        column_wall: asset_server.load("dungeon/frames/column_wall.png"),
        banner_blue: asset_server.load("dungeon/frames/wall_banner_blue.png"),
        banner_red: asset_server.load("dungeon/frames/wall_banner_red.png"),
        crate_texture: asset_server.load("dungeon/frames/crate.png"),
        door_closed: asset_server.load("dungeon/frames/doors_leaf_closed.png"),
        door_open: asset_server.load("dungeon/frames/doors_leaf_open.png"),
        tutorial_base: asset_server.load("dungeon/frames/column.png"),
        wizard_idle_frames: [
            asset_server.load("dungeon/frames/wizzard_m_idle_anim_f0.png"),
            asset_server.load("dungeon/frames/wizzard_m_idle_anim_f1.png"),
            asset_server.load("dungeon/frames/wizzard_m_idle_anim_f2.png"),
            asset_server.load("dungeon/frames/wizzard_m_idle_anim_f3.png"),
        ],
    });
}
