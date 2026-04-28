use glam::{Mat4, Vec3};
use gltf::camera::Projection;
use crate::renderer::framebuffer;

use super::{TextureCube, LightCube, Shader, Framebuffer, Texture2D, FullscreenQuad};

pub struct EnvironmentMap {
    cube: LightCube, // vertices for rendering purposes
    pub environment: TextureCube,
    pub irradiance: TextureCube,
    pub prefiltered: TextureCube,
    pub brdf_lut: Texture2D,
}

impl EnvironmentMap {
    pub fn new(path: &str, prefiltered_mips: u32) -> Self {
        // Build file paths
        let environment_path = format!("{}/environment.hdr", path);
        let irradiance_path = format!("{}/irradiance.hdr", path);
        let prefiltered_path = format!("{}/prefiltered", path);
        let brdf_lut_path = format!("{}/brdf_lut.png", path);

        // Load environment maps
        let cube = LightCube::new();
        let environment = TextureCube::from_hdr_equirectangular(&environment_path, 2048);
        let irradiance = TextureCube::from_hdr_equirectangular(&irradiance_path, 32);
        let prefiltered = TextureCube::from_dds(&prefiltered_path, 7);
        let brdf_lut = Texture2D::from_file(&brdf_lut_path, "RGB32F");

        Self { cube, environment, irradiance, prefiltered, brdf_lut }
    }

    pub fn draw_skybox(&self) {
        // Bind the environment texture
        self.environment.bind(0);

        // depth func and culling
        unsafe {
            gl::DepthFunc(gl::LEQUAL);
            gl::Disable(gl::CULL_FACE);
        }

        // cube
        self.cube.draw();

        // reset depth func and culling
        unsafe {
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
        }
    }

    pub fn bind_irradiance(&self, unit: u32) {
        self.irradiance.bind(unit);
    }

    pub fn bind_prefiltered(&self, unit: u32) {
        self.prefiltered.bind(unit);
    }

    pub fn bind_brdf_lut(&self, unit: u32) {
        self.brdf_lut.bind(unit);
    }
}
