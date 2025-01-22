mod braitenberg_vehicle;
mod light;
mod camera;
mod scene;
mod event_handlers;
mod gui;

use nannou_egui::{self, Egui};
use nannou::{color::srgb, event::Update, glam::Vec2, rand::random_range, App, Frame};
use braitenberg_vehicle::Vehicle;
use camera::Camera;
use light::Light;
use scene::{Scene, Scenes};

fn main() {
    nannou::app(Model::new)
        .update(Model::update)
        .run();
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
            .raw_event(event_handlers::raw_window_event)
            .mouse_wheel(event_handlers::handle_mouse_wheel)
            .key_released(event_handlers::handle_key_released)
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
        gui::update_gui(model, update);

        if model.paused {
            return;
        }
    
        for _ in 0..model.simulation_speed {
            let lights = if model.mouse_light { &model.lights } else { &model.lights[1..] };
            for vehicle in model.vehicles.iter_mut() {
                vehicle.update(lights, update.since_last.as_secs_f32());
            }
            model.replace_lights_on_collision();
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

    fn load_from_file(&mut self, scene: Scenes) {
        let scene = Scene::load_scene(scene);
        self.vehicles = scene.vehicles;
        let mouse_light = Light::new(Vec2::ZERO, srgb(1.0, 1.0, 1.0), 0.7);
        self.lights = vec![mouse_light];
        self.lights.extend(scene.lights);
        self.camera = scene.camera;
    }

    fn replace_lights_on_collision(&mut self) {
        self.lights.iter_mut().skip(self.mouse_light as usize)
            .filter_map(|light| 
                self.vehicles
                    .iter()
                    .find(|vehicle| {
                        let distance = light.position.distance_squared(vehicle.position);
                        distance < 20000.0 
                    })
                    .map(|vehicle| (light, vehicle)))
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
