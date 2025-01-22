mod braitenberg_vehicle;
mod light;
mod camera;
mod scene;

use nannou_egui::{self, egui::{self, Checkbox}, Egui};
use nannou::{color::srgb, event::{MouseScrollDelta, TouchPhase, Update}, glam::Vec2, rand::random_range, App, Frame};
use braitenberg_vehicle::Vehicle;
use camera::Camera;
use light::Light;
use scene::{Scene, Scenes};

fn main() {
    nannou::app(Model::new).update(Model::update).run();
}

struct Model {
    pub egui: Egui,
    vehicles: Vec<Vehicle>,
    lights: Vec<Light>, 
    camera: Camera,
    current_scene: Scenes,
    previous_scene: Scenes,
    show_controls: bool,
    simulation_speed: u32,
    follow_vehicle: bool,
    follow_vehicle_indx: usize,
    mouse_light: bool,
    paused: bool,
}

impl Model {
    fn new(app: &nannou::App) -> Self {
        let window_id = app.new_window()
            .size(800 as u32, 600 as u32)
            .view(Model::view)
            .raw_event(raw_window_event)
            .mouse_wheel(Model::handle_mouse_wheel)
            .key_released(Model::handle_key_released)
            .build()
            .unwrap();

        let window = app.window(window_id).unwrap();

        let mut model = Model {
            vehicles: Vec::new(),
            lights: Vec::new(),
            egui: Egui::from_window(&window),
            camera: Camera::new(),
            current_scene: Scenes::Scene1,
            previous_scene: Scenes::Scene1,
            show_controls: true,
            simulation_speed: 1,
            follow_vehicle: false,
            follow_vehicle_indx: 0,
            mouse_light: false,
            paused: false,
        };
        model.load_from_file(Scenes::Scene1);
        model
    }

    fn update(app: &App, model: &mut Self, update: Update) {
        model.camera.update_pos(&app.mouse);
        model.update_scene();
        model.update_mouse_light(app);
        model.update_gui(update);

        if model.paused {
            return;
        }
    
        for _ in 0..model.simulation_speed {
            let lights = if model.mouse_light { &model.lights } else { &model.lights[1..] };
            for vehicle in &mut model.vehicles {
                vehicle.update(lights, update.since_last.as_secs_f32());
            }
            Model::replace_lights_on_collision(model);
        }
        
        if model.follow_vehicle {
            model.camera.position = model.vehicles[model.follow_vehicle_indx].position;
        }
    }

    fn view(app: &App, model: &Self, frame: Frame) {
        let draw = app.draw();
        draw.background().color(nannou::color::BLACK);

        for light in model.get_lights() {
            light.draw(&draw, &model.camera);
        }

        for vehicle in &model.vehicles {
            vehicle.draw(&draw, &model.camera, model.get_lights());
        }

        draw.to_frame(app, &frame).unwrap();
        model.egui.draw_to_frame(&frame).unwrap();
    }

    fn update_scene(&mut self) {
        if self.current_scene != self.previous_scene {
            self.load_from_file(self.current_scene);
            self.previous_scene = self.current_scene;
            self.follow_vehicle_indx = 0;
        }
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

    fn load_from_file(&mut self, scene: Scenes) {
        let scene = Scene::load_scene(scene);
        self.vehicles = scene.vehicles;
        let mouse_light = Light::new(Vec2::ZERO, srgb(1.0, 1.0, 1.0), 0.7);
        self.lights = vec![mouse_light];
        self.lights.extend(scene.lights);
        self.camera = scene.camera;
    }

    pub fn update_gui(&mut self, update: Update) {
        self.egui.set_elapsed_time(update.since_start);
        let ctx = self.egui.begin_frame();
        egui::Window::new("Settings").show(&ctx, |ui: &mut egui::Ui| {
            ui.heading("Settings:");
            ui.label("Select Scene:");
            nannou_egui::egui::ComboBox::from_label("")
                .selected_text(self.current_scene.to_str())
                .show_ui(ui, |ui|{
                    for scene in Scenes::Scene1 as u8..=Scenes::Scene7 as u8 {
                        let scene = unsafe { std::mem::transmute(scene) };
                        ui.selectable_value(&mut self.current_scene, scene, scene.to_str());
                    }
                });
            if ui.add(egui::Button::new("Reset Scene")).clicked() {
                let scene = Scene::load_scene(self.current_scene);
                self.vehicles = scene.vehicles;
                let mouse_light = Light::new(Vec2::ZERO, srgb(1.0, 1.0, 1.0), 0.7);
                self.lights = vec![mouse_light];
                self.lights.extend(scene.lights);
            }
            ui.add(Checkbox::new(&mut self.show_controls, "Show Controls"));
            ui.add(Checkbox::new(&mut self.follow_vehicle, "Follow Vehicle"));
            if self.follow_vehicle && self.vehicles.len() > 1 {
                ui.label("Select Vehicle:");
                ui.horizontal(|ui| {
                    for i in 0..self.vehicles.len() {
                        if ui.add(egui::Button::new(format!("{i}"))).clicked() {
                            self.follow_vehicle_indx = i;
                        }                    
                    }
                });
            }
            ui.add(Checkbox::new(&mut self.mouse_light, "Show a light where the mouse is"));
            ui.add(egui::Slider::new(&mut self.simulation_speed, 1..=100).logarithmic(true).text("Simulation Speed"));
            ui.add(Checkbox::new(&mut self.paused, "Pause Simulation"));
            ui.label(format!("Camera Position: ({:.0}, {:.0})", self.camera.position.x, self.camera.position.y));
            ui.label(format!("Camera Zoom: {}", self.camera.zoom));            
        });

        if self.show_controls {
            egui::Window::new("Controlls").show(&ctx, |ui| {
                ui.label("- Right click and hold to move the camera.");
                ui.label("- Scroll to zoom in and out.");
                if ui.button("Close").clicked() {
                    self.show_controls = false;
                }
            });
        }
        
    }

    fn replace_lights_on_collision(model: &mut Model) {
        model.lights.iter_mut().skip(model.mouse_light as usize)
            .filter_map(|light| {
                let v = model.vehicles.iter().find(|vehicle| {
                    let distance = light.position.distance_squared(vehicle.position);
                    distance < 20000.0 
                });
                v.map(|vehicle| (light, vehicle))
            })
            .for_each(|(light, vehicle)| {
                light.position = nannou::geom::vec2(
                    vehicle.position.x  + random_range(-1000.0, 1000.0),
                    vehicle.position.y + random_range(-1000.0, 1000.0),
                );
            })
    }

    fn get_lights(&self) -> &[Light] {
        if self.mouse_light {
            &self.lights
        } else{
            &self.lights[1..]
        }
    }

    fn update_mouse_light(&mut self, app: &App) {
        let mouse_pos = nannou::geom::vec2(app.mouse.x, app.mouse.y);
        let mouse_pos_base_coords = mouse_pos / self.camera.zoom + self.camera.position;
        self.lights[0].position = mouse_pos_base_coords;
    }
}



fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}