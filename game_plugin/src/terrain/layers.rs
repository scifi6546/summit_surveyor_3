use super::{Terrain, TerrainPoint};
use bevy::prelude::*;
use slana::{GraphLayer, GridCoord};
#[derive(Debug)]
pub enum SpecialPoint {
    LiftBottom,
    LiftTop,
    /// What skiiers use to exit the ski resort
    ParkingLot,
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
