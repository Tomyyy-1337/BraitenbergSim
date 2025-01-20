use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::{braitenberg_vehicle::Vehicle, camera::Camera, light::Light};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Scenes {
    Scene1,
    Scene2,
    Scene3,
    Scene4,
    Scene5,
    Scene6,
}

impl Scenes {
    pub fn to_str(&self) -> &str {
        match self {
            Scenes::Scene1 => "Scene 1",
            Scenes::Scene2 => "Scene 2",
            Scenes::Scene3 => "Scene 3",
            Scenes::Scene4 => "Scene 4",
            Scenes::Scene5 => "Scene 5",
            Scenes::Scene6 => "Scene 6",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub vehicles: Vec<Vehicle>,
    pub lights: Vec<Light>,
    pub camera: Camera,
}

impl Scene {
    pub fn load_scene(scene:Scenes) -> Self {
        let file_path = match scene {
            Scenes::Scene1 => "scenes/scene1.json",
            Scenes::Scene2 => "scenes/scene2.json",
            Scenes::Scene3 => "scenes/scene3.json",
            Scenes::Scene4 => "scenes/scene4.json",
            Scenes::Scene5 => "scenes/scene5.json",
            Scenes::Scene6 => "scenes/scene6.json",
        };

        let file = File::open(file_path).unwrap();
        serde_json::from_reader(file).unwrap()
    }
}

