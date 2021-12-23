mod actions;
mod audio;
mod input;
mod loading;
mod menu;
mod player;
mod terrain;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::input::CameraInput;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;
use crate::terrain::TerrainPlugin;

use bevy::app::AppBuilder;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_state(GameState::Loading)
            .insert_resource(bevy_atmosphere::AtmosphereMat::default())
            .add_plugin(bevy_atmosphere::AtmospherePlugin { dynamic: false }) // Set to false since we aren't changing the sky's appearance
            .add_plugin(LoadingPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(TerrainPlugin)
            .add_plugin(CameraInput)
            .add_plugin(ActionsPlugin)
            .add_plugin(InternalAudioPlugin)
            .add_plugin(PlayerPlugin);

        #[cfg(debug_assertions)]
        {
            app.add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(LogDiagnosticsPlugin::default());
        }
    }
}
