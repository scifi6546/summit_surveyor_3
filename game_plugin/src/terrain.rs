use super::GameState;
use bevy::prelude::*;
use slana::Grid;
mod skiier;
pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Playing).with_system(build_terrain.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(skiier::build_skiiers.system())
                .with_system(skiier::skiier_path_follow.system()),
        );
    }
}
pub struct Terrain {
    grid: Grid<u32>,
}
impl Terrain {
    pub fn basic(size_x: u32, size_y: u32) -> Self {
        Self {
            grid: Grid::from_val((size_x, size_y), 2),
        }
    }
}
fn build_terrain(mut commands: Commands) {
    commands.spawn().insert(Terrain::basic(100, 100));
}
