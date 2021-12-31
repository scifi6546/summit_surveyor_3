mod decision;

use bevy::prelude::*;
use decision::{get_best_decision, DecisionResult};
use slana::{GraphLayer, GraphView, GridCoord, Path};
use std::{cmp::max, f32};
pub struct Skiier;
use super::{LiftLayer, ParkingLotLayer, SpecialPoint, Terrain, TerrainPoint, TrailCollection};
use bevy_mod_picking::PickableBundle;
#[derive(Debug, Clone, Copy)]
pub struct SkiierData {
    despawn_at_end: bool,
    total_cost: u32,
    /// prferred angle
    prefered_slope: f32,
    /// how much cost a skiier will endure before going home
    stamina: u32,
}
impl SkiierData {
    pub fn from_preferred_angle_stamina(angle: f32, stamina: u32) -> Self {
        let mut s = Self::default();
        s.prefered_slope = angle;
        s.stamina = stamina;
        s
    }
    /// adds skiier to self
    pub fn add(&self, skiier: &SkiierData) -> Self {
        Self {
            despawn_at_end: self.despawn_at_end || skiier.despawn_at_end,
            total_cost: self.total_cost + skiier.total_cost,
            prefered_slope: skiier.prefered_slope,
            stamina: skiier.stamina,
        }
    }
}
impl slana::WeightGetter<TerrainPoint, u32> for SkiierData {
    fn get_weight(&self, start: &TerrainPoint, end: &TerrainPoint) -> u32 {
        match *end {
            TerrainPoint::Ground { height } => {
                let end_height = height;
                match *start {
                    TerrainPoint::Ground { height } => {
                        let slope = (end_height - height).atan();

                        if slope < 0.0 {
                            let weight = (slope - self.prefered_slope).abs() * 10.0;
                            max(weight as u32, 3)
                        } else {
                            max((slope * 100.0) as u32, 5)
                        }
                    }
                    TerrainPoint::LiftBottom { up_cost } => up_cost,
                    TerrainPoint::LiftTop { up_cost } => up_cost,
                    TerrainPoint::ParkingLot => 1,
                    TerrainPoint::Trail => 1,
                }
            }
            TerrainPoint::LiftBottom { up_cost } => up_cost,
            TerrainPoint::LiftTop { up_cost } => up_cost,
            TerrainPoint::ParkingLot => 1,
            TerrainPoint::Trail => 1,
        }
    }
}
impl Default for SkiierData {
    fn default() -> Self {
        Self {
            despawn_at_end: false,
            total_cost: 0,
            stamina: 1000,
            prefered_slope: std::f32::consts::PI / 4.0,
        }
    }
}

use std::cmp::min;
const MAX_SKIIERS: usize = 10;
pub struct PathT {
    time: f32,
}
fn build_decision(
    view: &GraphView<TerrainPoint, SpecialPoint>,
    start: GridCoord,
    skiier_data: &SkiierData,
) -> (SkiierData, Path<u32>) {
    let (decisions, _new_skiier) = get_best_decision(&view, skiier_data, start);
    let (result, cost, mut path, mut current_skiier) =
        decisions[0].get_cost(&view, skiier_data, start);

    let mut end = path.get_end();
    let mut skiier = SkiierData::default();
    if result != DecisionResult::Despawn {
        for i in 1..decisions.len() {
            let (result, _, new_path, t_skiier) =
                decisions[i].get_cost(&view, &current_skiier, end);
            current_skiier = t_skiier;
            end = new_path.get_end();
            path.append(new_path);
            if result == DecisionResult::Despawn {
                skiier.despawn_at_end = true;
                break;
            }
        }
    } else {
        skiier.despawn_at_end = true;
    }
    return (current_skiier, path);
}
pub fn build_skiiers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    layer_query: QuerySet<(
        Query<&Terrain, ()>,
        Query<&ParkingLotLayer, ()>,
        Query<&LiftLayer, ()>,
        Query<&TrailCollection, ()>,
    )>,
    skiier_query: Query<(), With<Skiier>>,
) {
    let layers: Vec<&dyn GraphLayer<TerrainPoint, SpecialPoint = SpecialPoint>> = layer_query
        .q0()
        .iter()
        .map(|l| &l.grid as &dyn GraphLayer<TerrainPoint, SpecialPoint = SpecialPoint>)
        .chain(
            layer_query
                .q1()
                .iter()
                .map(|l| l as &dyn GraphLayer<TerrainPoint, SpecialPoint = SpecialPoint>),
        )
        .chain(
            layer_query
                .q2()
                .iter()
                .map(|l| l as &dyn GraphLayer<TerrainPoint, SpecialPoint = SpecialPoint>),
        )
        .chain(
            layer_query
                .q3()
                .iter()
                .map(|l| l as &dyn GraphLayer<TerrainPoint, SpecialPoint = SpecialPoint>),
        )
        .collect();

    let num_skiiers = skiier_query.iter().count();
    //   let view: GraphView<u32> = layers.into();
    let view = layers.into();
    let angles: [f32; MAX_SKIIERS] = [
        f32::consts::PI / 3.2,
        f32::consts::FRAC_PI_3,
        f32::consts::FRAC_PI_4,
        f32::consts::PI / 5.0,
        f32::consts::FRAC_PI_6,
        f32::consts::PI / 7.0,
        f32::consts::FRAC_PI_8,
        f32::consts::PI / 9.0,
        f32::consts::PI / 10.0,
        f32::consts::PI / 11.0,
    ];
    let stamina: [u32; MAX_SKIIERS] = [1000, 2000, 3000, 2000, 1501, 1234, 12341, 1245, 9123, 112];
    for i in 0..MAX_SKIIERS - num_skiiers {
        let (skiier, path) = build_decision(
            &view,
            GridCoord::from_xy(i as i32 % 5, 0),
            &SkiierData::from_preferred_angle_stamina(angles[i], stamina[i]),
        );
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                ..Default::default()
            })
            .insert_bundle(PickableBundle::default())
            .insert(Skiier)
            .insert(skiier)
            .insert(PathT { time: 0.0 })
            .insert(path);
    }
}
pub fn skiier_path_follow(
    mut commands: Commands,
    layer_query: QuerySet<(
        Query<&Terrain, ()>,
        Query<&ParkingLotLayer, ()>,
        Query<&LiftLayer, ()>,
        Query<&TrailCollection, ()>,
    )>,
    mut skiiers: Query<
        (
            Entity,
            &mut Path<u32>,
            &mut PathT,
            &mut Transform,
            &mut SkiierData,
        ),
        With<Skiier>,
    >,
) {
    let layers: Vec<&dyn GraphLayer<TerrainPoint, SpecialPoint = SpecialPoint>> = layer_query
        .q0()
        .iter()
        .map(|l| &l.grid as &dyn GraphLayer<TerrainPoint, SpecialPoint = SpecialPoint>)
        .chain(
            layer_query
                .q1()
                .iter()
                .map(|l| l as &dyn GraphLayer<TerrainPoint, SpecialPoint = SpecialPoint>),
        )
        .chain(
            layer_query
                .q2()
                .iter()
                .map(|l| l as &dyn GraphLayer<TerrainPoint, SpecialPoint = SpecialPoint>),
        )
        .chain(
            layer_query
                .q3()
                .iter()
                .map(|l| l as &dyn GraphLayer<TerrainPoint, SpecialPoint = SpecialPoint>),
        )
        .collect();
    let view = layers.into();
    let terrain = layer_query.q0().iter().next().unwrap();
    for (entity, mut path, mut path_time, mut transform, mut skiier_data) in skiiers.iter_mut() {
        if path.points.len() == 0 {
            continue;
        }
        path_time.time += 0.1;
        if path_time.time < path.points.len() as f32 - 1.0 {
            let idx = min(path_time.time.floor() as usize, path.points.len() - 1);
            let (x, y) = path.points[idx].to_xy();
            if idx < path.points.len() - 1 {
                let (x_next, y_next) = path.points[idx + 1].to_xy();
                let delta_time = path_time.time - idx as f32;
                let x_f = x_next as f32 * delta_time + (1.0 - delta_time) * x as f32;
                let z_f = y_next as f32 * delta_time + (1.0 - delta_time) * y as f32;
                let y_f = terrain.grid.interpolate(x_f, z_f);
                transform.translation.x = x_f;
                transform.translation.y = y_f;
                transform.translation.z = z_f;
            } else {
                transform.translation.x = x as f32;
                transform.translation.z = y as f32;
            }
        } else {
            if skiier_data.despawn_at_end {
                commands.entity(entity).despawn();
            } else {
                let (data, new_path) = build_decision(&view, path.get_end(), &skiier_data);
                *skiier_data = data;
                *path = new_path;
                path_time.time = 0.0;
            }
        }
    }
}
