use nannou::{event::{MouseScrollDelta, TouchPhase}, App};

use crate::{scene::Scenes, Model};

pub fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

pub fn handle_mouse_wheel(_app: &App, model: &mut Model, delta: MouseScrollDelta, _state: TouchPhase) {
    match delta {
        MouseScrollDelta::LineDelta(_x, y) => model.camera.update_zoom(y),
        MouseScrollDelta::PixelDelta(_pos) => {}
    }
}

pub fn handle_key_released(_app: &App, model: &mut Model, key: nannou::event::Key) {
    match key {
        nannou::event::Key::Key1 => model.current_scene = Scenes::Scene1,
        nannou::event::Key::Key2 => model.current_scene = Scenes::Scene2,
        nannou::event::Key::Key3 => model.current_scene = Scenes::Scene3,
        nannou::event::Key::Key4 => model.current_scene = Scenes::Scene4,
        nannou::event::Key::Key5 => model.current_scene = Scenes::Scene5,
        nannou::event::Key::Key6 => model.current_scene = Scenes::Scene6,
        nannou::event::Key::Key7 => model.current_scene = Scenes::Scene7,
        nannou::event::Key::Space => model.paused = !model.paused,
        _ => {}
    }
}