use geom::Tri;
use nannou::prelude::*;
use serde::{Deserialize, Serialize};

use crate::camera::Camera;

#[derive(Serialize, Deserialize)]
pub struct Light {
    pub position: Vec2,
    pub color: nannou::color::rgb::Rgb,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vec2, color: nannou::color::rgb::Rgb, intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
        }
    }

    pub fn draw(&self, draw: &nannou::draw::Draw, camera: &Camera) {
        self.draw_circle(draw, camera, 100.0, (0.0, 0.8));
        self.draw_circle(draw, camera, 400.0, (0.0, 0.1));
        self.draw_circle(draw, camera, 800.0, (0.0, 0.05));

    }
        
    fn draw_circle(&self, draw: &Draw, camera: &Camera, radius: f32, color_range: (f32, f32)) {
        let num_triangles = 40;
        let center = Vec2::new(self.position.x - camera.position.x, self.position.y - camera.position.y) * camera.zoom;
        
        let tris = (0..num_triangles).map(|i| {
            let angle1 = map_range(i, 0, num_triangles, 0.0, 2.0 * PI);
            let angle2 = map_range(i + 1, 0, num_triangles, 0.0, 2.0 * PI);
        
            let x1 = self.position.x + radius * angle1.cos() - camera.position.x;
            let y1 = self.position.y + radius * angle1.sin() - camera.position.y;
            let x2 = self.position.x + radius * angle2.cos() - camera.position.x;
            let y2 = self.position.y + radius * angle2.sin() - camera.position.y;
        
            geom::Tri([
                center,
                Vec2::new(x1, y1) * camera.zoom,
                Vec2::new(x2, y2) * camera.zoom,
            ])
        }).map(|tri| {
            tri.map_vertices(|v| {
                let color = if v == center {
                    srgba(self.color.red, self.color.green, self.color.blue, self.intensity * color_range.1)
                } else {
                    srgba(0.0, 0.0, 0.0, 0.0)
                };
                (v.extend(0.0), color)
            })
        });
        
        draw.mesh().tris_colored(tris);
    }


}

