use gl::MAX;
use gltf::Camera;
use glam::{Mat4,Vec3};

use super::{Buffer, Light};

// std140 structs
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SettingsUniforms {
    gamma: f32,
    exposure: f32,
    environment: u32,
    rendermode: u32,
    ssao_radius: f32,
    ssao_bias: f32,
    tonemap: u32, //actually a bool
    _padding: [f32; 1],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CameraUniforms {
    view_matrix: [[f32; 4]; 4],
    projection_matrix: [[f32; 4]; 4],
    camera_position: [f32; 3],
    _padding: f32, // padding to make size multiple of 16 bytes
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LightsUniforms {
    count: [u32; 4], // count.x is the number of lights, rest is padding
    lights: [Light; 1], // array of 1 light
}


pub struct UboManager {
    settings: SettingsUniforms,
    settings_ubo: Buffer,
    settings_dirty: bool,

    camera: CameraUniforms,
    camera_ubo: Buffer,
    camera_dirty: bool, 

    lights: LightsUniforms,
    lights_ubo: Buffer,
    lights_dirty: bool,
}

impl UboManager {
    pub fn new() -> Self {
        // settings
        let settings = SettingsUniforms {
            gamma: 2.2,
            exposure: 1.0,
            environment: 0,
            rendermode: 0,
            ssao_radius: 0.5,
            ssao_bias: 0.025,
            tonemap: 1,
            _padding: [0.0; 1],
        };
        let settings_ubo = Buffer::new();
        settings_ubo.allocate::<SettingsUniforms>(std::mem::size_of::<SettingsUniforms>(), gl::DYNAMIC_DRAW);
        settings_ubo.write(&[settings]);

        // camera
        let camera = CameraUniforms {
            view_matrix: [[0.0; 4]; 4],
            projection_matrix: [[0.0; 4]; 4],
            camera_position: [0.0; 3],
            _padding: 0.0,
        };
        let camera_ubo = Buffer::new();
        camera_ubo.allocate::<CameraUniforms>(std::mem::size_of::<CameraUniforms>(), gl::DYNAMIC_DRAW);
        camera_ubo.write(&[camera]);

        // lights
        let lights = LightsUniforms {
            count: [0; 4],
            lights: [Light::new([0.0; 3], 0.0, [0.0; 3], 0.0)],
        };
        let lights_ubo = Buffer::new();
        lights_ubo.allocate::<LightsUniforms>(std::mem::size_of::<LightsUniforms>(), gl::DYNAMIC_DRAW);
        lights_ubo.write(&[lights]);

        // bind settings and camera UBOs
        settings_ubo.bind_base(0); // Binding point 0 for settings
        camera_ubo.bind_base(1); // Binding point 1 for camera
        lights_ubo.bind_base(2); // Binding point 2 for lights

        Self {
            settings,
            settings_ubo,
            settings_dirty: false,
            camera,
            camera_ubo,
            camera_dirty: false,
            lights,
            lights_ubo,
            lights_dirty: false,
        }
    }
    
    // Settings
    pub fn set_settings(
        &mut self, 
        (gamma, exposure, environment, rendermode, ssao_radius, ssao_bias, tonemap): (
            f32, f32, u32, u32, f32, f32, bool
        )
    ) {
        self.settings.gamma = gamma;
        self.settings.exposure = exposure;
        self.settings.environment = environment;
        self.settings.rendermode = rendermode;
        self.settings.ssao_radius = ssao_radius;
        self.settings.ssao_bias = ssao_bias;
        self.settings.tonemap = if tonemap { 1 } else { 0 };
        self.settings_dirty = true;
    }

    // Camera
    pub fn set_camera(
        &mut self, 
        (view_matrix, projection_matrix, camera_position): (
            Mat4,
            Mat4,
            Vec3
        ),
    ) {
        self.camera.view_matrix = view_matrix.to_cols_array_2d();
        self.camera.projection_matrix = projection_matrix.to_cols_array_2d();
        self.camera.camera_position = camera_position.to_array();
        self.camera_dirty = true;
    }

    // Lights
    // set one light
    pub fn set_light(&mut self, index: usize, light: Light) {
        if index < self.lights.lights.len() {
            self.lights.lights[index] = light;
            self.lights.count[0] = self.lights.lights.len() as u32;
            self.lights_dirty = true;
        }
    }
    // set all lights
    pub fn set_lights(&mut self, lights: Vec<Light>) {
        let count = lights.len().min(MAX as usize);
        for i in 0..count {
            self.lights.lights[i] = lights[i];
        }
        self.lights.count[0] = count as u32;
        self.lights_dirty = true;
    }

    // Update UBOs if dirty
    pub fn update(&mut self) {
        if self.settings_dirty {
            self.settings_ubo.write(&[self.settings]);
            self.settings_dirty = false;
        }
        if self.camera_dirty {
            self.camera_ubo.write(&[self.camera]);
            self.camera_dirty = false;
        }
        if self.lights_dirty {
            self.lights_ubo.write(&[self.lights]);
            self.lights_dirty = false;
        }
    }
}