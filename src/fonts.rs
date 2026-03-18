use bevy::prelude::*;

pub struct FontPlugin;

#[derive(Resource)]
pub struct GameFonts {
    pub pixel_regular: Handle<Font>,
    pub pixel_bold: Handle<Font>,
}

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameFonts>();
    }
}

impl FromWorld for GameFonts {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>().clone();

        Self {
            pixel_regular: asset_server.load("silkscreen_regular.ttf"),
            pixel_bold: asset_server.load("silkscreen_bold.ttf"),
        }
    }
}
