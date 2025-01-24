use nannou::{glam::Vec2, state::Mouse};
use serde::{Deserialize, Serialize};

pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    last_active_mouse_pos: Vec2,
    is_active: bool,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vec2::new(0.0, 0.0),
            zoom: 1.0,
            last_active_mouse_pos: Vec2::new(0.0, 0.0),
            is_active: false,
        }
    }

    pub fn update_pos(&mut self, mouse: &Mouse) {
        if self.is_active {
            let delta = self.last_active_mouse_pos - mouse.position();
            self.position += delta / self.zoom;
        } 
        self.is_active = mouse.buttons.right().is_down();
        self.last_active_mouse_pos = mouse.position();
    }

    pub fn update_zoom(&mut self, delta: f32) {
        self.zoom *= 1.0 + delta * 0.2;
        self.zoom = self.zoom.max(0.02);
    }
}

#[derive(Deserialize, Serialize)]
struct CameraDef {
    position: Vec2,
    zoom: f32,
}

impl<'de> Deserialize<'de> for Camera {
    fn deserialize<D>(deserializer: D) -> Result<Camera, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let CameraDef { position, zoom } = CameraDef::deserialize(deserializer)?;
        Ok(Camera {
            position,
            zoom,
            last_active_mouse_pos: Vec2::new(0.0, 0.0),
            is_active: false,
        })
    }
}

impl Serialize for Camera {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let def = CameraDef {
            position: self.position,
            zoom: self.zoom,
        };
        def.serialize(serializer)
    }
}