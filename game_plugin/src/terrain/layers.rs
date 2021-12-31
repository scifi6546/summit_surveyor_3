use super::{Terrain, TerrainPoint};
use bevy::{
    prelude::*,
    render::{mesh::Indices, pipeline::PrimitiveTopology},
};
use slana::{GraphLayer, GridCoord};
mod trails;
pub use trails::TrailCollection;
#[derive(Debug)]
pub enum SpecialPoint {
    LiftBottom,
    LiftTop,
    /// What skiiers use to exit the ski resort
    ParkingLot,
    Trail,
}
pub struct LiftLayer {
    top: GridCoord,
    bottom: GridCoord,
    up_cost: u32,
}

impl GraphLayer<TerrainPoint> for LiftLayer {
    type SpecialPoint = SpecialPoint;
    fn get_special_pooints(&self) -> Vec<(Self::SpecialPoint, GridCoord)> {
        vec![
            (SpecialPoint::LiftTop, self.top),
            (SpecialPoint::LiftBottom, self.bottom),
        ]
    }
    fn get_children(&self, coord: GridCoord) -> Vec<(GridCoord, TerrainPoint)> {
        if coord == self.bottom {
            vec![(
                self.top,
                TerrainPoint::LiftTop {
                    up_cost: self.up_cost,
                },
            )]
        } else {
            vec![]
        }
    }
    fn get_node(&self, coord: GridCoord) -> Option<TerrainPoint> {
        if coord != self.bottom && coord != self.top {
            None
        } else if coord == self.bottom {
            Some(TerrainPoint::LiftBottom {
                up_cost: self.up_cost,
            })
        } else {
            Some(TerrainPoint::LiftTop {
                up_cost: self.up_cost,
            })
        }
    }
}
pub fn build_lift(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    terrain: &Terrain,
    bottom_x: i32,
    bottom_y: i32,
    top_x: i32,
    top_y: i32,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.1, 0.5, 0.2).into()),
            transform: Transform::from_xyz(
                bottom_x as f32,
                match terrain.grid.get(bottom_x, bottom_y) {
                    TerrainPoint::Ground { height } => *height,
                    _ => panic!("invalid point type"),
                },
                bottom_y as f32,
            ),
            ..Default::default()
        })
        .insert(LiftLayer {
            top: GridCoord::from_xy(top_x, top_y),
            bottom: GridCoord::from_xy(bottom_x, bottom_y),
            up_cost: 2,
        });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.1, 0.2, 0.8).into()),
        transform: Transform::from_xyz(
            top_x as f32,
            match terrain.grid.get(top_x, top_y) {
                TerrainPoint::Ground { height } => *height,
                _ => panic!("invalid point type"),
            },
            top_y as f32,
        ),
        ..Default::default()
    });
}
/// Graph Layer representing a parking lot. SKiiers use this to exit the stage
pub struct ParkingLotLayer {
    position: GridCoord,
}
impl GraphLayer<TerrainPoint> for ParkingLotLayer {
    type SpecialPoint = SpecialPoint;
    fn get_special_pooints(&self) -> Vec<(Self::SpecialPoint, GridCoord)> {
        vec![(SpecialPoint::ParkingLot, self.position)]
    }
    fn get_children(&self, _coord: GridCoord) -> Vec<(GridCoord, TerrainPoint)> {
        vec![]
    }
    fn get_node(&self, coord: GridCoord) -> Option<TerrainPoint> {
        if coord != self.position {
            None
        } else {
            Some(TerrainPoint::ParkingLot)
        }
    }
}
pub fn build_parkinglot(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    terrain: &Terrain,
    x: i32,
    y: i32,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.15, 0.1, 0.1).into()),
            transform: Transform::from_xyz(
                x as f32,
                match terrain.grid.get(x, y) {
                    TerrainPoint::Ground { height } => *height,
                    _ => panic!("invalid point type"),
                },
                y as f32,
            ),
            ..Default::default()
        })
        .insert(ParkingLotLayer {
            position: GridCoord::from_xy(x, y),
        });
}
pub fn build_trails(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    terrain: &Terrain,
) {
    let mut trails = TrailCollection::default();
    let (s, e) = trails.add_trail(
        GridCoord::from_xy(10, 10),
        10.0,
        GridCoord::from_xy(50, 50),
        10.0,
    );
    trails
        .append_trail(e, GridCoord::from_xy(60, 60), 10.0)
        .expect("failed to add");
    trails
        .append_trail(e, GridCoord::from_xy(50, 40), 10.0)
        .expect("failed to add");
    for (coord, radius) in trails.iter_trails() {
        let (x, y) = coord.to_xy();
        let z = match terrain.grid.get(x, y) {
            TerrainPoint::Ground { height } => *height,
            _ => panic!("invalid point type"),
        };
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.15, 0.1, 0.1).into()),
            transform: Transform::from_xyz(x as f32, z, y as f32),
            ..Default::default()
        });
    }
    for (start, end) in trails.iter_paths() {
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(build_trail_mesh(
                Vec3::new(start.x, terrain.grid.interpolate(start.x, start.y), start.y),
                Vec3::new(end.x, terrain.grid.interpolate(end.x, end.y), end.y),
            )),
            material: materials.add(Color::rgb(0.15, 0.1, 0.1).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        });
    }
    commands.spawn().insert(trails);
}
fn build_trail_mesh(start: Vec3, end: Vec3) -> Mesh {
    let slope = (start.z - end.z) / (start.x - end.x);
    let theta = slope.atan();
    info!("theta: {}", theta);
    let dx1 = theta.sin();
    let dy1 = theta.cos();
    let theta_2 = (slope + std::f32::consts::FRAC_PI_2).atan();
    let dx2 = theta_2.sin();
    let dy2 = theta_2.cos();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let position = vec![
        [start.x + dx1, start.y, start.z + dy1],
        [start.x + dx2, start.y, start.z + dy2],
        [end.x + dx1, end.y, end.z + dy1],
        [end.x + dx2, end.y, end.z + dy2],
    ];
    let indicies = vec![0, 2, 1, 1, 2, 0, 1, 2, 3, 3, 2, 1];
    let normals = vec![
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    let uvs = vec![[1.0, 1.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]];
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, position);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indicies)));
    return mesh;
}
