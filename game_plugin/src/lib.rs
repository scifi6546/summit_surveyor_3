mod actions;
mod audio;
mod input;
mod loading;

mod camera;
mod terrain;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::input::CameraInput;
use crate::loading::LoadingPlugin;

use crate::camera::CameraPlugin;
use crate::terrain::TerrainPlugin;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_mod_picking::*;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    Loading,
    // During this State the actual game logic is executed
    Playing,
}
pub mod prelude {
    pub use super::terrain::TerrainPickingSet;
}
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state(GameState::Loading)
            .add_plugin(PickingPlugin)
            .add_plugin(bevy_mod_picking::DefaultPickingPlugins)
            .add_plugin(InteractablePickingPlugin)
            .add_plugin(HighlightablePickingPlugin)
            //.insert_resource(bevy_atmosphere::AtmosphereMat::default())
            //.add_plugin(bevy_atmosphere::AtmospherePlugin { dynamic: false }) // Set to false since we aren't changing the sky's appearance
            .add_plugin(LoadingPlugin)
            .add_plugin(TerrainPlugin)
            .add_plugin(CameraInput)
            .add_plugin(ActionsPlugin)
            .add_plugin(InternalAudioPlugin)
            //    .add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new())
            .add_plugin(CameraPlugin);

        #[cfg(debug_assertions)]
        {
            app.add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(LogDiagnosticsPlugin::default());
        }
    }
}
