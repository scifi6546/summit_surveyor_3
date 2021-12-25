use super::GameState;
use bevy::{
    prelude::*,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
};
use nalgebra::Vector3;
use slana::{GraphLayer, Grid, GridCoord};
mod skiier;
pub struct TerrainPlugin;
pub struct LiftLayer {
    top: GridCoord,
    bottom: GridCoord,
    up_cost: u32,
}
impl GraphLayer<u32> for LiftLayer {
    fn get_children(&self, coord: GridCoord) -> Vec<(GridCoord, u32)> {
        if coord == self.bottom {
            vec![(self.top, self.up_cost)]
        } else {
            vec![]
        }
    }
}
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
    fn build_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut position = vec![];
        let mut normals = vec![];
        let mut uvs = vec![];
        for x in 0..self.grid.size().0 - 1 {
            for y in 0..self.grid.size().1 - 1 {
                let x0_y0 = Vector3::new(
                    x as f32,
                    *self.grid.get(x as i32, y as i32) as f32,
                    y as f32,
                );
                let x0_y1 = Vector3::new(
                    x as f32,
                    *self.grid.get(x as i32, y as i32 + 1) as f32,
                    y as f32 + 1.0,
                );
                let x1_y0 = Vector3::new(
                    x as f32 + 1.0,
                    *self.grid.get(x as i32 + 1, y as i32) as f32,
                    y as f32,
                );
                let x1_y1 = Vector3::new(
                    x as f32 + 1.0,
                    *self.grid.get(x as i32 + 1, y as i32 + 1) as f32,
                    y as f32 + 1.0,
                );
                let triangle0_normal = (x0_y1 - x0_y0).cross(&(x1_y0 - x0_y0)).normalize();
                let triangle1_normal = (x1_y0 - x1_y1).cross(&(x0_y1 - x1_y1)).normalize();
                //vert 0
                position.push([x0_y0.x, x0_y0.y, x0_y0.z]);
                normals.push([triangle0_normal.x, triangle0_normal.y, triangle0_normal.z]);
                uvs.push([0.0, 0.0]);
                //vert 1
                position.push([x0_y1.x, x0_y1.y, x0_y1.z]);
                normals.push([triangle0_normal.x, triangle0_normal.y, triangle0_normal.z]);
                uvs.push([0.0, 1.0]);
                //vert 2
                position.push([x1_y0.x, x1_y0.y, x1_y0.z]);
                normals.push([triangle0_normal.x, triangle0_normal.y, triangle0_normal.z]);
                uvs.push([1.0, 0.0]);
                //Triangle 1
                //vert3
                position.push([x1_y1.x, x1_y1.y, x1_y1.z]);
                normals.push([triangle1_normal.x, triangle1_normal.y, triangle1_normal.z]);
                uvs.push([1.0, 0.0]);
                //vert4
                position.push([x1_y0.x, x1_y0.y, x1_y0.z]);
                normals.push([triangle1_normal.x, triangle1_normal.y, triangle1_normal.z]);
                uvs.push([1.0, 1.0]);
                //vert5
                position.push([x0_y1.x, x0_y1.y, x0_y1.z]);
                normals.push([triangle1_normal.x, triangle1_normal.y, triangle1_normal.z]);
                uvs.push([0.0, 1.0]);
            }
        }
        let indicies = (0..position.len()).map(|i| i as u32).collect();
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, position);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indicies)));
        return mesh;
    }
}
fn build_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let terrain = Terrain::basic(100, 100);
    let mesh = terrain.build_mesh();
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.1, 0.5, 0.2).into()),
            ..Default::default()
        })
        .insert(Terrain::basic(100, 100));
    let lift_top_x = 5;
    let lift_top_y = 5;
    let lift_bottom_x = 0;
    let lift_bottom_y = 0;
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.1, 0.5, 0.2).into()),
            transform: Transform::from_xyz(lift_bottom_x as f32, 0.0, lift_bottom_y as f32),
            ..Default::default()
        })
        .insert(LiftLayer {
            top: GridCoord::from_xy(lift_top_x, lift_top_y),
            bottom: GridCoord::from_xy(lift_bottom_x, lift_bottom_y),
            up_cost: 2,
        });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.1, 0.5, 0.2).into()),
        transform: Transform::from_xyz(lift_top_x as f32, 0.0, lift_top_y as f32),
        ..Default::default()
    });
}
