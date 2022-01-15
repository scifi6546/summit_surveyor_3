use bevy::{
    input::{mouse::MouseWheel, prelude::*},
    prelude::*,
};
use smooth_bevy_cameras::controllers::orbit::{ControlEvent, OrbitCameraController};
pub struct CameraInput;
impl Plugin for CameraInput {
    fn build(&self, app: &mut App) {
        //      app.add_system(default_input_map);
    }
}
/*
pub fn default_input_map(
    mut events: EventWriter<ControlEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    controllers: Query<&OrbitCameraController>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().next() {
        controller
    } else {
        return;
    };
    let OrbitCameraController {
        enabled,
        mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        mouse_wheel_zoom_sensitivity,
        ..
    } = *controller;

    if !enabled {
        return;
    }

    let cursor_delta = Vec2::ZERO;
    /*
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }
    */
    if keyboard.pressed(KeyCode::Left) {
        events.send(ControlEvent::Orbit(
            mouse_rotate_sensitivity * Vec2::new(-4.0, 0.0),
        ))
    }
    if keyboard.pressed(KeyCode::Right) {
        events.send(ControlEvent::Orbit(
            mouse_rotate_sensitivity * Vec2::new(4.0, 0.0),
        ))
    }
    if keyboard.pressed(KeyCode::Up) {
        events.send(ControlEvent::Orbit(
            mouse_rotate_sensitivity * Vec2::new(0.0, 4.0),
        ))
    }
    if keyboard.pressed(KeyCode::Down) {
        events.send(ControlEvent::Orbit(
            mouse_rotate_sensitivity * Vec2::new(0.0, -4.0),
        ))
    }
    if mouse_buttons.pressed(MouseButton::Left) {
        events.send(ControlEvent::Orbit(mouse_rotate_sensitivity * cursor_delta));
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        events.send(ControlEvent::TranslateTarget(
            mouse_translate_sensitivity * cursor_delta,
        ));
    }

    let mut scalar = 1.0;
    for event in mouse_wheel_reader.iter() {
        scalar *= 1.0 + -event.y * mouse_wheel_zoom_sensitivity;
    }
    events.send(ControlEvent::Zoom(scalar));
}
*/
