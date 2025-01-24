mod braitenberg_vehicle;
mod light;
mod camera;
mod scene;
mod event_handlers;
mod gui;

use nannou_egui::{self, Egui};
use nannou::{color::srgb, event::Update, glam::Vec2, rand::random_range, App, Draw, Frame};
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
    draw_background: bool,
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
            draw_background: true,
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
        
        model.draw_background(&draw, app);

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

    fn draw_background(&self, draw: &Draw, app: &App) {
        let grid_color_1 = srgb(0.0, 0.0, 0.0); 
        draw.background().color(grid_color_1);
        if !self.draw_background {
            return;
        }
        let grid_color_2 = srgb(0.05, 0.05, 0.05); 
        let square_size = 500.0;
        let zoom_adjusted_size = square_size * self.camera.zoom;
        let window_size = app.window_rect().wh();
        let offset = Vec2::new(
            (self.camera.position.x * self.camera.zoom).rem_euclid(zoom_adjusted_size),
            (self.camera.position.y * self.camera.zoom).rem_euclid(zoom_adjusted_size),
        );

        let mut x_width_tiles = window_size.x.div_euclid(zoom_adjusted_size);
        let mut y_width_tiles = window_size.y.div_euclid(zoom_adjusted_size);

        x_width_tiles -= x_width_tiles.rem_euclid(2.0) - 2.0;
        y_width_tiles -= y_width_tiles.rem_euclid(2.0) - 2.0;

        let start_pos = -offset - Vec2::new(x_width_tiles * zoom_adjusted_size, y_width_tiles * zoom_adjusted_size) / 2.0;
        let end_pos = window_size / 2.0 + zoom_adjusted_size;
        
        let mut x = start_pos.x;
        let mut y = start_pos.y;
        let mut alternator = ((self.camera.position.y * self.camera.zoom).rem_euclid(zoom_adjusted_size * 2.0) < zoom_adjusted_size) ^ ((self.camera.position.x * self.camera.zoom).rem_euclid(zoom_adjusted_size * 2.0) < zoom_adjusted_size);
        while y < end_pos.y {
            alternator = !alternator;
            let mut alternator = alternator;
            while x < end_pos.x {
                alternator = !alternator;
                if alternator {
                    draw.rect()
                        .x_y(x, y)
                        .w_h(zoom_adjusted_size, zoom_adjusted_size)
                        .color(grid_color_2);
                }
                x += zoom_adjusted_size
            }
            x = start_pos.x;
            y += zoom_adjusted_size;
        }
    }
}
