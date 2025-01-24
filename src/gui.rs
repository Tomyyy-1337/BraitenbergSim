use nannou::{color::srgb, event::Update, glam::Vec2};
use nannou_egui::egui::{self, Checkbox};

use crate::{light::Light, scene::{Scene, Scenes}, Model};

pub fn update_gui(model: &mut Model, update: Update) {
    model.egui.set_elapsed_time(update.since_start);
    let ctx = model.egui.begin_frame();
    egui::Window::new("Settings").show(&ctx, |ui: &mut egui::Ui| {
        ui.heading("Settings:");
        ui.label("Select Scene:");
        nannou_egui::egui::ComboBox::from_label("")
            .selected_text(model.current_scene.to_str())
            .show_ui(ui, |ui|{
                for scene in Scenes::Scene1 as u8..=Scenes::Scene7 as u8 {
                    let scene = unsafe { std::mem::transmute(scene) };
                    ui.selectable_value(&mut model.current_scene, scene, scene.to_str());
                }
            });
        if ui.add(egui::Button::new("Reset Scene")).clicked() {
            let scene = Scene::load_scene(model.current_scene);
            model.vehicles = scene.vehicles;
            let mouse_light = Light::new(Vec2::ZERO, srgb(1.0, 1.0, 1.0), 0.7);
            model.lights = vec![mouse_light];
            model.lights.extend(scene.lights);
        }
        ui.add(Checkbox::new(&mut model.show_controls, "Show Controls"));
        ui.add(Checkbox::new(&mut model.follow_vehicle, "Follow Vehicle"));
        if model.follow_vehicle && model.vehicles.len() > 1 {
            ui.label("Select Vehicle:");
            ui.horizontal(|ui| {
                for i in 0..model.vehicles.len() {
                    if ui.add(egui::Button::new(format!("{i}"))).clicked() {
                        model.follow_vehicle_indx = i;
                    }                    
                }
            });
        }
        ui.add(Checkbox::new(&mut model.mouse_light, "Show a light at Mouse Position"));
        ui.label("Simulation Speed:");
        ui.add(egui::Slider::new(&mut model.simulation_speed, 1..=100).logarithmic(true));
        ui.add(Checkbox::new(&mut model.paused, "Pause Simulation"));
        ui.add(Checkbox::new(&mut model.draw_background, "Show background grid"));
        ui.label(format!("Camera Position: ({:.0}, {:.0})", model.camera.position.x, model.camera.position.y));
        ui.label(format!("Camera Zoom: {}", model.camera.zoom));            
    });

    if model.show_controls {
        egui::Window::new("Controlls").show(&ctx, |ui| {
            ui.label("- Right click and hold to move the camera.");
            ui.label("- Scroll to zoom in and out.");
            if ui.button("Close").clicked() {
                model.show_controls = false;
            }
        });
    }
}