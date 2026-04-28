use glam::{Vec2, Vec3};
use rand::Rng;
use super::{Framebuffer, Texture2D, Shader, FullscreenQuad};

pub struct SSAOPass {
    pub ssao_framebuffer: Framebuffer,
    pub ssao_blur_framebuffer: Framebuffer,
    noise_texture: Texture2D,
    kernel: Vec<Vec3>,
    ssao_shader: Shader,
    ssao_blur_shader: Shader,
    quad: FullscreenQuad,
}

impl SSAOPass {
    pub fn new(width: u32, height: u32) -> Self {
        // initialize
        let mut ssao_framebuffer = Framebuffer::new(width, height);
        let ssao_texture = Texture2D::empty(width, height, gl::R16F, gl::LINEAR, gl::REPEAT);
        ssao_framebuffer.add_color_attachment(ssao_texture);
        ssao_framebuffer.check_complete();

        let mut ssao_blur_framebuffer = Framebuffer::new(width, height);
        let ssao_blur_texture = Texture2D::empty(width, height, gl::R16F, gl::LINEAR, gl::REPEAT);
        ssao_blur_framebuffer.add_color_attachment(ssao_blur_texture);
        ssao_blur_framebuffer.check_complete();

        let ssao_shader = Shader::from_files(
            "assets/shaders/quad.vertex.glsl",
            "assets/shaders/ssao.fragment.glsl",
        );
        let ssao_blur_shader = Shader::from_files(
            "assets/shaders/quad.vertex.glsl",
            "assets/shaders/ssao_blur.fragment.glsl",
        );

        let quad = FullscreenQuad::new();
        // noise texture
        let mut rng = rand::rng();

        let mut noise: Vec<Vec3> = Vec::new();
        for _ in 0..16 {
            noise.push(Vec3::new(
                rng.random_range(-1.0..1.0),
                rng.random_range(-1.0..1.0),
                0.0,
            ));
        }

        let noise_texture = Texture2D::from_bytes(
            4, 4,
            gl::RGB16F, gl::RGB, 
            gl::NEAREST, gl::REPEAT, 
            &noise,
        );
        // the kernel
        let mut kernel = Vec::with_capacity(64);
        for i in 0..64 {
            let mut sample = Vec3::new(
                rng.random_range(-1.0..1.0),
                rng.random_range(-1.0..1.0),
                rng.random_range(0.0..1.0),
            );
            sample = sample.normalize();
            
            let scale = i as f32 / 64.0;
            let scale = 0.1 + 0.9 * scale * scale;
            sample *= scale;

            kernel.push(sample);
        }

        SSAOPass {
            ssao_framebuffer,
            ssao_blur_framebuffer,
            noise_texture,
            kernel,
            ssao_shader,
            ssao_blur_shader,
            quad,
        }
    }

    pub fn draw(&mut self, normal_texture: Texture2D, depth_texture: Texture2D) {
        // SSAO PASS
        self.ssao_framebuffer.bind();
        self.ssao_shader.bind();

        depth_texture.bind(0);
        normal_texture.bind(1);
        self.noise_texture.bind(2);

        self.ssao_shader.set_vec3c("samples", 64, &self.kernel);
        self.ssao_shader.set_int("screen_width", self.ssao_framebuffer.width as i32);
        self.ssao_shader.set_int("screen_height", self.ssao_framebuffer.height as i32);

        self.quad.draw();
        // SSAO BLUR PASS
        self.ssao_blur_framebuffer.bind();
        self.ssao_blur_shader.bind();

        self.ssao_framebuffer.color[0].bind(0);
        depth_texture.bind(1);

        self.ssao_blur_shader.set_int("ao_texture", 0);
        self.ssao_blur_shader.set_int("depth_texture", 1);

        self.ssao_blur_shader.set_vec2(
            "texelSize",
            &Vec2::new(
                1.0 / self.ssao_blur_framebuffer.width as f32,
                1.0 / self.ssao_blur_framebuffer.height as f32,
            ),
        );

        self.quad.draw();
    }
}