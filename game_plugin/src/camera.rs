use crate::GameState;
use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
pub struct CameraPlugin;
use super::prelude::TerrainPickingSet;
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastMethod, RayCastSource, RaycastSystem};

pub struct Player;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LookTransformPlugin)
            .add_plugin(OrbitCameraPlugin::new(false))
            .add_plugin(DefaultRaycastingPlugin::<TerrainPickingSet>::default())
            .add_startup_system(spawn_camera.system())
            .add_system_to_stage(CoreStage::PostUpdate, print_events.system())
            .add_system_set(
                SystemSet::on_enter(GameState::Playing).with_system(spawn_light.system()),
            )
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(print_events))
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_system(update_raycast_with_cursor)
                    .before(RaycastSystem::BuildRays),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(show_intersection.system())
                    .after(RaycastSystem::BuildRays),
            );
    }
}
pub fn print_events(mut events: EventReader<bevy_mod_picking::PickingEvent>) {
    for event in events.iter() {
        info!("This event happened! {:?}", event);
    }
}
#[derive(Component)]
struct IntersectSphere;
// Update our `RayCastSource` with the current cursor position every frame.
fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RayCastSource<TerrainPickingSet>>,
) {
    for mut pick_source in &mut query.iter_mut() {
        // Grab the most recent cursor event if it exists:
        if let Some(cursor_latest) = cursor.iter().last() {
            pick_source.cast_method = RayCastMethod::Screenspace(cursor_latest.position);
        }
    }
}
fn spawn_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            PerspectiveCameraBundle::default(),
            Vec3::new(-2.0, 5.0, 5.0),
            Vec3::new(0., 0., 0.),
        ))
        .insert_bundle(bevy_mod_picking::PickingCameraBundle::default())
        .insert(RayCastSource::<TerrainPickingSet>::new()); // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 2,
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..Default::default()
        })
        .insert(IntersectSphere);
}

fn spawn_light(mut commands: Commands) {
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(1.0, 1.0, 1.0).into(),
            intensity: 8000.0,
            range: 500.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(50.0, 40.0, 50.0),
        ..Default::default()
    });
}
fn show_intersection(
    mut sphere_query: Query<&mut Transform, With<IntersectSphere>>,
    source_query: Query<&RayCastSource<TerrainPickingSet>, ()>,
) {
    if let Some(cast_source) = source_query.iter().next() {
        if let Some((_entity, intersect)) = cast_source.intersect_top() {
            for mut sphere in sphere_query.iter_mut() {
                sphere.translation = intersect.position();
            }
        }
    }
}
