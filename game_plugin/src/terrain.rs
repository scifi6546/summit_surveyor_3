use super::GameState;
mod layers;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use layers::{
    build_lift, build_parkinglot, build_trails, LiftLayer, ParkingLotLayer, SpecialPoint,
    TrailCollection,
};
use nalgebra::Vector3;
use slana::Grid;
mod skiier;
pub struct TerrainPlugin;
use bevy_mod_raycast::RayCastMesh;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Playing).with_system(build_terrain.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(skiier::build_skiiers)
                .with_system(skiier::skiier_path_follow),
        );
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TerrainPoint {
    Ground { height: f32 },
    LiftBottom { up_cost: u32 },
    LiftTop { up_cost: u32 },
    ParkingLot,
    Trail,
}
impl slana::ToF32 for TerrainPoint {
    fn to_f32(&self) -> f32 {
        match *self {
            Self::Ground { height } => height,
            _ => panic!("invalid point type: {:#?}", self),
        }
    }
}
pub struct TerrainPickingSet;
#[derive(Component)]
pub struct Terrain {
    pub grid: Grid<TerrainPoint, SpecialPoint>,
}
impl Terrain {
    pub fn basic(size_x: u32, size_y: u32) -> Self {
        Self {
            grid: Grid::from_val((size_x, size_y), TerrainPoint::Ground { height: 2.0 }),
        }
    }
    fn output() -> Self {
        Self::from_pgm(include_str!("../../heightmaps/output.pgm").to_string())
    }
    fn cone() -> Self {
        Self::from_pgm(include_str!("../../heightmaps/cone.pgm").to_string())
    }
    pub fn from_pgm(string: String) -> Self {
        Self {
            grid: slana::importer::terrain_from_pgm(string)
                .expect("failed to parse")
                .convert(|u| TerrainPoint::Ground {
                    height: u as f32 / 1000.0,
                }),
        }
    }
    pub fn slope(size_x: u32, size_y: u32) -> Self {
        Self {
            grid: Grid::from_fn((size_x, size_y), |x, y| TerrainPoint::Ground {
                height: x as f32 + 1.0,
            }),
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
                    match self.grid.get(x as i32, y as i32) {
                        TerrainPoint::Ground { height } => *height,
                        _ => panic!("invalid point type"),
                    },
                    y as f32,
                );
                let x0_y1 = Vector3::new(
                    x as f32,
                    match self.grid.get(x as i32, y as i32 + 1) {
                        TerrainPoint::Ground { height } => *height,
                        _ => panic!("invalid point type"),
                    },
                    y as f32 + 1.0,
                );
                let x1_y0 = Vector3::new(
                    x as f32 + 1.0,
                    match self.grid.get(x as i32 + 1, y as i32) {
                        TerrainPoint::Ground { height } => *height,
                        _ => panic!("invalid point type"),
                    },
                    y as f32,
                );
                let x1_y1 = Vector3::new(
                    x as f32 + 1.0,
                    match self.grid.get(x as i32 + 1, y as i32 + 1) {
                        TerrainPoint::Ground { height } => *height,
                        _ => panic!("invalid point type"),
                    },
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
    // let terrain = Terrain::slope(100, 100);
    //  let terrain = Terrain::output();
    let terrain = Terrain::cone();
    //let terrain = Terrain::basic(100, 100);

    build_lift(
        &mut commands,
        &mut meshes,
        &mut materials,
        &terrain,
        50,
        40,
        60,
        60,
    );
    build_lift(
        &mut commands,
        &mut meshes,
        &mut materials,
        &terrain,
        1,
        1,
        50,
        50,
    );
    build_parkinglot(&mut commands, &mut meshes, &mut materials, &terrain, 10, 10);
    build_trails(&mut commands, &mut meshes, &mut materials, &terrain);
    let mesh = terrain.build_mesh();
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.8, 0.1, 0.8).into()),
            ..Default::default()
        })
        .insert_bundle(bevy_mod_picking::PickableBundle::default())
        .insert(terrain)
        .insert(RayCastMesh::<TerrainPickingSet>::default());
}
