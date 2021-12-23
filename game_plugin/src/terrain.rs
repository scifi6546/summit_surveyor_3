use super::GameState;
use bevy::prelude::*;
use ndarray::Array2;
pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Playing).with_system(build_terrain.system()),
        );
    }
}
struct Terrain {
    grid: Array2<f32>,
}
impl Terrain {
    pub fn basic(size_x: u32, size_y: u32) -> Self {
        Self {
            grid: Array2::from_elem((size_x, size_y), 1.0),
        }
    }
}
fn build_terrain(commands: &mut Commands) {
    commands.spawn().insert(Terrain::basic(100, 100));
}
fn dijkstra() {}
