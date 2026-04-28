use glam::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Light { // so far we assume only point lights
    pub position: [f32; 3],
    pub range: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}

impl Light {
    pub fn new(position: [f32; 3], range: f32, color: [f32; 3], intensity: f32) -> Self {
        Self {
            position,
            range,
            color,
            intensity,
        }
    }
}

pub struct LightManager {
    lights: Vec<Light>,
}

impl LightManager {
    pub fn new() -> Self {
        Self {
            lights: Vec::new(),
        }
    }

    pub fn add_light(&mut self, position: [f32; 3], range: f32, color: [f32; 3], intensity: f32) {
        let light = Light::new(position, range, color, intensity);
        self.lights.push(light);
    }
    pub fn set_light(&mut self, index: usize, light: Light) {
        if index < self.lights.len() {
            self.lights[index] = light;
        }
    }

    pub fn get_lights(&self) -> Vec<Light> {
        self.lights.clone()
    }
    pub fn get_light(&self, index: usize) -> Option<Light> {
        self.lights.get(index).cloned()
    }

    // get all lights position in an array of vec3
    pub fn get_light_positions(&self) -> Vec<Vec3> {
        self.lights.iter().map(|light| Vec3::from(light.position)).collect()
    }
}