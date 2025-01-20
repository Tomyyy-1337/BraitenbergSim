mod braitenberg_vehicle;
mod light;
mod camera;
mod scene;

use nannou_egui::{self, egui::{self, Checkbox}, Egui};
use nannou::{color::srgb, event::{MouseScrollDelta, TouchPhase, Update}, glam::Vec2, rand::random_range, state::mouse, App, Frame};
use braitenberg_vehicle::{Vehicle, VehicleType};
use camera::Camera;
use light::Light;
use scene::{Scene, Scenes};

fn main() {
    let mut lights = Vec::new();
    for _ in 0..100 {
        let color = srgb(random_range(0.0, 1.0), random_range(0.0, 1.0), random_range(0.0, 1.0));
        let x = random_range(-10000.0, 10000.0);
        let y = random_range(-10000.0, 10000.0);
        let light = Light::new(Vec2::new(x, y), color, 0.7);
        lights.push(light);
    } 
    let scene = Scene {
        vehicles: vec![
            Vehicle::new(VehicleType::TwoA, Vec2::new(-600.0, 0.0)),
            Vehicle::new(VehicleType::TwoB, Vec2::new(-200.0, 0.0)),
            Vehicle::new(VehicleType::ThreeA, Vec2::new(200.0, 0.0)),
            Vehicle::new(VehicleType::ThreeB, Vec2::new(600.0, 0.0)),
        ],
        lights,
        camera: Camera::new(),
    };
    // write to file
    let path = "scenes/scene6.json";
    let json = serde_json::to_string(&scene).unwrap();
    std::fs::write(path, json).unwrap();


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
    mouse_light_active: bool,
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
            mouse_light_active: false,
        };
        model.load_from_file(Scenes::Scene1);
        model
    }

    fn update(app: &App, model: &mut Self, update: Update) {
        model.camera.update_pos(&app.mouse);
        model.update_scene();
        
        if model.mouse_light && !model.mouse_light_active {
            let light = Light::new(Vec2::ZERO, srgb(1.0, 1.0, 1.0), 1.0);
            model.lights.push(light);
            model.mouse_light_active = true;
        } else if !model.mouse_light && model.mouse_light_active {
            model.lights.pop();
            model.mouse_light_active = false;
        }
        if model.mouse_light_active {
            let mouse_pos = nannou::geom::vec2(app.mouse.x, app.mouse.y);
            let mouse_pos_base_coords = mouse_pos / model.camera.zoom + model.camera.position;
            model.lights.last_mut().unwrap().position = mouse_pos_base_coords;
        }

        for _ in 0..model.simulation_speed {
            for vehicle in &mut model.vehicles {
                vehicle.update(&model.lights, update.since_last.as_secs_f32());
            }
    
            Model::replace_lights_on_collision(model);
        }

        if model.follow_vehicle {
            let vehicle = &model.vehicles[model.follow_vehicle_indx];
            model.camera.position = vehicle.position;
        }

        model.update_gui(update);
    }

    fn view(app: &App, model: &Self, frame: Frame) {
        let draw = app.draw();
        draw.background().color(nannou::color::BLACK);

        for light in &model.lights {
            light.draw(&draw, &model.camera);
        }

        for vehicle in &model.vehicles {
            vehicle.draw(&draw, &model.camera, &model.lights);
        }

        draw.to_frame(app, &frame).unwrap();
        model.egui.draw_to_frame(&frame).unwrap();
    }

    fn update_scene(&mut self) {
        if self.current_scene != self.previous_scene {
            self.load_from_file(self.current_scene);
            self.previous_scene = self.current_scene;
            self.mouse_light_active = false;
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
            _ => {}
        }
    }

    fn load_from_file(&mut self, scene: Scenes) {
        let scene = Scene::load_scene(scene);
        self.vehicles = scene.vehicles;
        self.lights = scene.lights;
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
                    ui.selectable_value(&mut self.current_scene, Scenes::Scene1, Scenes::Scene1.to_str());
                    ui.selectable_value(&mut self.current_scene, Scenes::Scene2, Scenes::Scene2.to_str());
                    ui.selectable_value(&mut self.current_scene, Scenes::Scene3, Scenes::Scene3.to_str());
                    ui.selectable_value(&mut self.current_scene, Scenes::Scene4, Scenes::Scene4.to_str());
                    ui.selectable_value(&mut self.current_scene, Scenes::Scene5, Scenes::Scene5.to_str());
                    ui.selectable_value(&mut self.current_scene, Scenes::Scene6, Scenes::Scene6.to_str());
                });
            if ui.add(egui::Button::new("Reset Scene")).clicked() {
                let scene = Scene::load_scene(self.current_scene);
                self.vehicles = scene.vehicles;
                self.lights = scene.lights;
                self.camera = scene.camera;
            }
            ui.add(Checkbox::new(&mut self.show_controls, "Show Controls"));
            ui.add(Checkbox::new(&mut self.follow_vehicle, "Follow Vehicle"));
            if self.follow_vehicle {
                ui.add(egui::Slider::new(&mut self.follow_vehicle_indx, 0..=(self.vehicles.len()- 1)).text("Vehicle Index"));
            }
            ui.add(Checkbox::new(&mut self.mouse_light, "Show a light where the mouse is"));
            ui.add(egui::Slider::new(&mut self.simulation_speed, 1..=100).text("Simulation Speed"));
            ui.label(format!("Camera Position: ({:.0}, {:.0})", self.camera.position.x, self.camera.position.y));
            ui.label(format!("Camera Zoom: {}", self.camera.zoom));            
        });

        if self.show_controls {
            egui::Window::new("Controlls").show(&ctx, |ui| {
                ui.label("- Right click and drag to move the camera.");
                ui.label("- Scroll to zoom in and out.");
                ui.label("- 1-4 to switch between scenes.");
                if ui.button("Close").clicked() {
                    self.show_controls = false;
                }
            });
        }
        
    }

    fn replace_lights_on_collision(model: &mut Model) {
        model.lights.iter_mut()
            .filter(|light| {
                model.vehicles.iter().any(|vehicle| {
                    let distance = light.position.distance_squared(vehicle.position);
                    distance < 20000.0
                })
            })
            .for_each(|light| {
                light.position = nannou::geom::vec2(
                    random_range(-1000.0, 1000.0),
                    random_range(-1000.0, 1000.0),
                );
            })
    }
}


fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}