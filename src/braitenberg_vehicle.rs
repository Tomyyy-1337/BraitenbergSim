use std::f32::consts::{FRAC_PI_2, PI};

use nannou::{color::{self, srgb, Srgb}, draw::{self}, glam::Vec2, math::{Vec2Angle, Vec2Rotate}, prelude::Pow, text::layout::DEFAULT_Y_ALIGN};
use serde::{Deserialize, Serialize};

use crate::{camera::Camera, light::Light};

const VEHICLE_WIDTH: f32 = 60.0;
const VEHICLE_LENGTH: f32 = 100.0;
const SENSOR_SIZE: f32 = 10.0;

#[derive(Serialize, Deserialize)]
pub enum VehicleType {
    TwoA,
    TwoB,
    ThreeA,
    ThreeB,
}

#[derive(Serialize, Deserialize)]
pub struct Vehicle {
    vehicle_type: VehicleType,
    pub position: Vec2,
    pub orientation: f32,
    pub velocity: f32,
}

impl Vehicle {
    pub fn new(vehicle_type: VehicleType, position: Vec2) -> Self {
        Vehicle {
            vehicle_type,
            position,
            orientation: 0.0,
            velocity: 0.0,
        }
    }

    pub fn update(&mut self, lights: &[Light], delta: f32) {
        let left_sensor_value = self.read_sensor(Vec2::from(LEFT_FRONT), lights);
        let right_sensor_value = self.read_sensor(Vec2::from(RIGHT_FRONT), lights);
        match self.vehicle_type {
            VehicleType::TwoA => self.two_sensor_vehicle(
                delta,
                left_sensor_value,
                right_sensor_value 
            ),
            VehicleType::TwoB => self.two_sensor_vehicle( 
                delta,
                right_sensor_value,
                left_sensor_value
            ),
            VehicleType::ThreeA => self.two_sensor_vehicle( 
                delta,
                (1.0 - left_sensor_value).max(0.0),
                (1.0 - right_sensor_value).max(0.0)
            ),
            VehicleType::ThreeB => self.two_sensor_vehicle(
                delta,
                (1.0 - right_sensor_value).max(0.0),
                (1.0 - left_sensor_value).max(0.0)
            ),
        }
    }

    pub fn draw(&self, draw: &nannou::draw::Draw, camera: &Camera, lights: &[Light]) {
        let color = match self.vehicle_type {
            VehicleType::TwoA => srgb(1.0, 1.0, 0.0),
            VehicleType::TwoB => srgb(0.0, 1.0, 1.0),
            VehicleType::ThreeA => srgb(0.0, 0.0, 1.0),
            VehicleType::ThreeB => srgb(1.0, 0.0, 1.0),
        };

        self.draw_rect(color, Vec2::new(0.0, 0.0), draw, camera, Vec2::new(VEHICLE_WIDTH, VEHICLE_LENGTH));

        match self.vehicle_type {
            VehicleType::TwoA => VehicleA::draw(self, draw, camera, lights, srgb(0.0, 1.0, 0.0)),
            VehicleType::TwoB => VehicleB::draw(self, draw, camera, lights, srgb(0.0, 1.0, 0.0)),
            VehicleType::ThreeA => VehicleA::draw(self, draw, camera, lights, srgb(1.0, 0.0, 0.0)),
            VehicleType::ThreeB => VehicleB::draw(self, draw, camera, lights, srgb(1.0, 0.0, 0.0)),
        }
    }

    pub fn to_global_cords(&self, offset: Vec2, camera: &Camera) -> Vec2 {
        let offset = offset.rotate(self.orientation);
        (self.position + offset - camera.position) * camera.zoom
    }

    pub fn draw_rect(&self, color: Srgb, offset: Vec2, draw: &nannou::draw::Draw, camera: &Camera, dimensions: Vec2) -> Vec2 {
        let pos = self.to_global_cords(offset, camera);

        draw.rect()
            .x_y(pos.x, pos.y)
            .w_h( dimensions.x * camera.zoom, dimensions.y * camera.zoom)
            .color(color)
            .rotate(self.orientation);

        pos
    }

    pub fn draw_line(&self, start: Vec2, end: Vec2, stroke: f32, draw: &nannou::draw::Draw, camera: &Camera, color: Srgb) {
        draw.line()
            .start(start)
            .end(end)
            .stroke_weight(stroke * camera.zoom)
            .color(color);
    }

    pub fn draw_circle(&self, pos: Vec2, radius: f32, draw: &nannou::draw::Draw, camera: &Camera, color: Srgb) {
        draw.ellipse()
            .x_y(pos.x, pos.y)
            .radius(radius * camera.zoom)
            .color(color)
            .resolution(100.0)
            .rotate(self.orientation);
    }

    pub fn read_sensor(&self, pos: Vec2, lights: &[Light]) -> f32 {
        let pos = pos.rotate(self.orientation) + self.position;
        
        let val: f32 = lights.iter()
            .map(|light| {
                let dist = light.position.distance_squared(pos);
                light.intensity/dist
            })
            .sum(); 
        (val * 20000.0).min(1.0)
    }

    fn two_sensor_vehicle(&mut self, delta: f32, left_sensor_value: f32, right_sensor_value: f32) {        
        let new_vel = (left_sensor_value + right_sensor_value) * 1600.0;
        let new_vel_min = new_vel.min(600.0);

        self.velocity = new_vel_min;

        let factor = (new_vel_min / new_vel).min(1.0);

        let rotation = self.calc_rotation(left_sensor_value, right_sensor_value, factor);
        self.orientation += rotation * delta;
        
        self.position += Vec2::new(-self.orientation.sin(), self.orientation.cos()) * self.velocity * delta;
    }

    fn calc_rotation(&self,left_sensor_value: f32, right_sensor_value: f32, factor: f32) -> f32 {
        if left_sensor_value < right_sensor_value {
            return -self.calc_rotation(right_sensor_value, left_sensor_value, factor);
        }

        if right_sensor_value == 0.0 {
            return 0.0;
        }

        let h = VEHICLE_WIDTH + VEHICLE_WIDTH / (left_sensor_value / right_sensor_value - 1.0);
        let r = (left_sensor_value).atan2(h) * -10000.0;
        r * factor
    }
    

}

const LEFT_FRONT: (f32, f32) = (-VEHICLE_WIDTH / 2.0, VEHICLE_LENGTH / 2.0);
const RIGHT_FRONT: (f32, f32) = (VEHICLE_WIDTH / 2.0, VEHICLE_LENGTH / 2.0);
struct VehicleA; 
impl VehicleA {
    pub fn draw(vehicle: &Vehicle, draw: &draw::Draw, camera: &Camera, lights: &[Light], color: Srgb) {
        let sensor_size = 10.0;
        
        let front_left = vehicle.draw_rect(srgb(1.0, 0.0, 0.0), Vec2::new((-VEHICLE_WIDTH + sensor_size) / 2.0, (VEHICLE_LENGTH + sensor_size) / 2.0), draw, camera, Vec2::new(SENSOR_SIZE, SENSOR_SIZE));
        let front_right = vehicle.draw_rect(srgb(1.0, 0.0, 0.0), Vec2::new((VEHICLE_WIDTH - sensor_size) / 2.0, (VEHICLE_LENGTH + sensor_size) / 2.0), draw, camera, Vec2::new(SENSOR_SIZE, SENSOR_SIZE));
        
        let back_left = vehicle.to_global_cords(Vec2::new((-VEHICLE_WIDTH + sensor_size) / 2.0, -(VEHICLE_LENGTH - sensor_size) / 2.0), camera);
        let back_right = vehicle.to_global_cords(Vec2::new((VEHICLE_WIDTH - sensor_size) / 2.0, -(VEHICLE_LENGTH - sensor_size) / 2.0), camera);

        let left_wheel = vehicle.draw_rect(srgb(1.0, 0.0, 0.0), Vec2::new((-VEHICLE_WIDTH - 10.0)  / 2.0, -(VEHICLE_LENGTH - sensor_size) / 2.0), draw, camera, Vec2::new(10.0, 20.0));
        let right_wheel = vehicle.draw_rect(srgb(1.0, 0.0, 0.0), Vec2::new((VEHICLE_WIDTH + 10.0) / 2.0, -(VEHICLE_LENGTH - sensor_size) / 2.0), draw, camera, Vec2::new(10.0, 20.0));

        let left_sensor_val = (vehicle.read_sensor(Vec2::from(LEFT_FRONT), lights) * 25.0).log10();
        let right_sensor_val = (vehicle.read_sensor(Vec2::from(RIGHT_FRONT), lights) * 25.0).log10();
        let left_color = srgb(left_sensor_val * color.red, left_sensor_val * color.green, left_sensor_val * color.blue);
        let right_color = srgb(right_sensor_val * color.red, right_sensor_val * color.green, right_sensor_val * color.blue);

        vehicle.draw_line(front_left, back_left, 5.0, draw, camera, left_color);
        vehicle.draw_line(front_right, back_right, 5.0,  draw, camera, right_color);

        vehicle.draw_circle(back_left, 2.5, draw, camera, left_color);
        vehicle.draw_circle(back_right, 2.5, draw, camera, right_color);

        vehicle.draw_line(left_wheel, back_left, 5.0, draw, camera, left_color);
        vehicle.draw_line(right_wheel, back_right, 5.0, draw, camera, right_color);
    }
}

struct VehicleB;
impl VehicleB {
    pub fn draw(vehicle: &Vehicle, draw: &draw::Draw, camera: &Camera, lights: &[Light], color: Srgb) {
        let sensor_size = 10.0;
        
        let front_left = vehicle.draw_rect(srgb(1.0, 0.0, 0.0), Vec2::new((-VEHICLE_WIDTH + sensor_size) / 2.0, (VEHICLE_LENGTH + sensor_size) / 2.0), draw, camera, Vec2::new(SENSOR_SIZE, SENSOR_SIZE));
        let front_right = vehicle.draw_rect(srgb(1.0, 0.0, 0.0), Vec2::new((VEHICLE_WIDTH - sensor_size) / 2.0, (VEHICLE_LENGTH + sensor_size) / 2.0), draw, camera, Vec2::new(SENSOR_SIZE, SENSOR_SIZE));
        
        let back_left = vehicle.to_global_cords(Vec2::new((-VEHICLE_WIDTH + sensor_size) / 2.0, -(VEHICLE_LENGTH - sensor_size) / 2.0), camera);
        let back_right = vehicle.to_global_cords(Vec2::new((VEHICLE_WIDTH - sensor_size) / 2.0, -(VEHICLE_LENGTH - sensor_size) / 2.0), camera);

        let front_left_center = (front_left * 2.0 + back_left) / 3.0;
        let front_right_center = (front_right * 2.0 + back_right) / 3.0;

        let back_left_center = (front_left + back_left * 2.0) / 3.0;
        let back_right_center = (front_right + back_right * 2.0) / 3.0;

        let left_wheel = vehicle.draw_rect(srgb(1.0, 0.0, 0.0), Vec2::new((-VEHICLE_WIDTH - 10.0)  / 2.0, -(VEHICLE_LENGTH - sensor_size) / 2.0), draw, camera, Vec2::new(10.0, 20.0));
        let right_wheel = vehicle.draw_rect(srgb(1.0, 0.0, 0.0), Vec2::new((VEHICLE_WIDTH + 10.0) / 2.0, -(VEHICLE_LENGTH - sensor_size) / 2.0), draw, camera, Vec2::new(10.0, 20.0));

        let left_sensor_val = (vehicle.read_sensor(Vec2::from(LEFT_FRONT), lights) * 25.0).log10();
        let right_sensor_val = (vehicle.read_sensor(Vec2::from(RIGHT_FRONT), lights) * 25.0).log10();
        let left_color = srgb(left_sensor_val * color.red, left_sensor_val * color.green, left_sensor_val * color.blue);
        let right_color = srgb(right_sensor_val * color.red, right_sensor_val * color.green, right_sensor_val * color.blue);

        vehicle.draw_line(front_left, front_left_center, 5.0, draw, camera, left_color);
        vehicle.draw_line(front_right, front_right_center, 5.0, draw, camera, right_color);
        
        vehicle.draw_line(front_left_center, back_right_center, 5.0, draw, camera, left_color);
        vehicle.draw_line(front_right_center, back_left_center, 5.0, draw, camera, right_color);

        vehicle.draw_line(back_right_center, back_right, 5.0, draw, camera, left_color);
        vehicle.draw_line(back_left_center, back_left, 5.0, draw, camera, right_color);

        vehicle.draw_line(right_wheel, back_right, 5.0, draw, camera, left_color);
        vehicle.draw_line(left_wheel, back_left, 5.0, draw, camera, right_color);

        vehicle.draw_circle(back_right, 2.5, draw, camera, left_color);
        vehicle.draw_circle(back_left, 2.5, draw, camera, right_color);

        vehicle.draw_circle(front_left_center, 2.5, draw, camera, left_color);
        vehicle.draw_circle(front_right_center, 2.5, draw, camera, right_color);

        vehicle.draw_circle(back_right_center, 2.5, draw, camera, left_color);
        vehicle.draw_circle(back_left_center, 2.5, draw, camera, right_color);
    }
}