mod decision;
use bevy::prelude::*;
use decision::get_best_decision;
use slana::{dijkstra, GraphLayer, GraphView, GridCoord, Path};
pub struct Skiier;
use super::{LiftLayer, SpecialPoint, Terrain};
use std::{
    cmp::{min, Reverse},
    collections::BinaryHeap,
};
const MAX_SKIIERS: usize = 1;
pub struct PathT {
    time: f32,
}

pub fn build_skiiers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    skiier_query: Query<(), With<Skiier>>,
    terrain: Query<&Terrain, ()>,
    lift_query: Query<&LiftLayer, ()>,
) {
    let layers: Vec<&dyn GraphLayer<u32, SpecialPoint = SpecialPoint>> = terrain
        .iter()
        .map(|terrain| &terrain.grid as &dyn GraphLayer<u32, SpecialPoint = SpecialPoint>)
        .chain(
            lift_query
                .iter()
                .map(|l| l as &dyn GraphLayer<u32, SpecialPoint = SpecialPoint>),
        )
        .collect();

    let num_skiiers = skiier_query.iter().count();
    //   let view: GraphView<u32> = layers.into();
    let view = layers.into();
    for i in 0..MAX_SKIIERS - num_skiiers {
        info!("spawning {} skiier", i);
        let decisions = get_best_decision(&view, GridCoord::from_xy(i as i32 % 5, 0));
        let (r, _, mut path) = decisions[0].get_cost(&view, GridCoord::from_xy(i as i32 % 5, 0));
        info!("decision result: {:#?}", r);
        let mut end = path.get_end();
        for i in 1..decisions.len() {
            let (r, _, new_path) = decisions[i].get_cost(&view, end);
            info!("decision result: {:#?}", r);
            end = new_path.get_end();
            path.append(new_path);
        }
        info!("decsisions: {}", decisions.len());
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                ..Default::default()
            })
            .insert(Skiier)
            .insert(PathT { time: 0.0 })
            .insert(path);
    }
}
pub fn skiier_path_follow(
    time: Res<Time>,
    mut skiiers: Query<(&Path, &mut PathT, &mut Transform), With<Skiier>>,
) {
    for (path, mut path_time, mut transform) in skiiers.iter_mut() {
        if path.points.len() == 0 {
            continue;
        }
        path_time.time += 1.0 * time.delta_seconds();
        let idx = min(path_time.time.floor() as usize, path.points.len() - 1);
        let (x, y) = path.points[idx].to_xy();
        if idx < path.points.len() - 1 {
            let (x_next, y_next) = path.points[idx + 1].to_xy();
            let delta_time = path_time.time - idx as f32;
            transform.translation.x = x_next as f32 * delta_time + (1.0 - delta_time) * x as f32;
            transform.translation.z = y_next as f32 * delta_time + (1.0 - delta_time) * y as f32;
        } else {
            transform.translation.x = x as f32;
            transform.translation.z = y as f32;
        }
    }
}
