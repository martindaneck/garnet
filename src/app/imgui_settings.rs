use std::ffi::CString;
use imgui::*;
use imgui_opengl_renderer::Renderer;
use glfw::{Action, Context, Key};

use crate::renderer::light::Light;

pub struct Settings {
    gamma: f32,
    exposure: f32,
    rendermode: u32,
    environment: u32,
    ssao_radius: f32,
    ssao_bias: f32,
    tonemap: bool,
    light1: Light,
}

pub struct ImguiSettings {
    imgui: imgui::Context,
    renderer: Renderer,
    settings: Settings,
    last_frame_time: std::time::Instant,
}

impl ImguiSettings {
    pub fn new() -> Self {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        imgui.style_mut().use_dark_colors();

        //initial io
        let io = imgui.io_mut();
        io.display_size = [800.0, 600.0];
        io.delta_time = 1.0 / 60.0;

        let renderer = Renderer::new(&mut imgui, |s| {
            // glfwGetProcAddress REQUIRES A NULL TERMINATED STRING, OTHERWISE UB
            let c_str = CString::new(s).unwrap();
            unsafe {
                glfw::ffi::glfwGetProcAddress(c_str.as_ptr() as *const i8)
                    .map(|f| f as *const _)
                    .unwrap()   
            }
        });

        let settings = Settings {
            gamma: 2.2,
            exposure: 1.0,
            rendermode: 0,
            environment: 0,
            ssao_radius: 0.271,
            ssao_bias: 0.015,
            tonemap: true,
            light1: Light::new([0.0, 3.0, 0.0], 10.0, [1.0, 1.0, 1.0], 1.0),
        };

        Self { imgui, renderer, settings, last_frame_time: std::time::Instant::now() }
    }

    pub fn begin_frame(&mut self, window: &mut glfw::PWindow) {
        let io = self.imgui.io_mut();
        // update time
        let now = std::time::Instant::now();
        io.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        // update display size
        let (width, height) = window.get_framebuffer_size();
        io.display_size = [width as f32, height as f32];
        // update mouse position
        let (x, y) = window.get_cursor_pos();
        io.mouse_pos = [x as f32, y as f32];
        // update mouse buttons
        io.mouse_down[0] = window.get_mouse_button(glfw::MouseButtonLeft) == Action::Press;
        io.mouse_down[1] = window.get_mouse_button(glfw::MouseButtonRight) == Action::Press;
    }

    pub fn draw(&mut self, fps: u32) {
        let ui = self.imgui.frame();

        ui.window("Settings")
            .size([300.0, 800.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Press \'F3\' to toggle ImGui");
                ui.text("Press \'E\' to lock/unlock mouse");
                ui.text("Press \'ESC\' to exit app");
                ui.text(&format!("FPS:  {}", fps));
                ui.separator();
                ui.text("HDR");
                ui.checkbox("Gamma Correct and Tonemap", &mut self.settings.tonemap);
                ui.slider("Gamma", 0.1, 4.0, &mut self.settings.gamma);
                ui.slider("Exposure", 0.01, 10.0, &mut self.settings.exposure);
                ui.separator();
                ui.text("SSAO");
                ui.slider("Radius", 0.0, 1.0, &mut self.settings.ssao_radius);
                ui.slider("Bias", 0.0, 0.1, &mut self.settings.ssao_bias);
                ui.separator();
                ui.text("Light 1");
                ui.slider("Position X", -5.0, 5.0, &mut self.settings.light1.position[0]);
                ui.slider("Position Y", -5.0, 5.0, &mut self.settings.light1.position[1]);
                ui.slider("Position Z", -5.0, 5.0, &mut self.settings.light1.position[2]);
                ui.slider("Range", 0.0, 100.0, &mut self.settings.light1.range);
                ui.color_edit3("Color", &mut self.settings.light1.color);
                ui.slider("Intensity", 0.0, 100.0, &mut self.settings.light1.intensity);
                ui.separator();
                ui.text("Environment:");
                ui.radio_button("Environment: Fireplace", &mut self.settings.environment, 0);
                ui.radio_button("Environment: Sky", &mut self.settings.environment, 1);
                ui.radio_button("Environment: Meadow", &mut self.settings.environment, 2);
                ui.radio_button("Environment: Space", &mut self.settings.environment, 3);
                ui.separator();
                ui.text("Render Mode (Debug)");
                ui.radio_button("Render Mode: Default", &mut self.settings.rendermode, 0);
                ui.radio_button("Render Mode: Albedo Map", &mut self.settings.rendermode, 1);
                ui.radio_button("Render Mode: Normal Map", &mut self.settings.rendermode, 2);
                ui.radio_button("Render Mode: ORM Map", &mut self.settings.rendermode, 3);
                ui.radio_button("Render Mode: Vertex Normal", &mut self.settings.rendermode, 4);
                ui.radio_button("Render Mode: Vertex Tangent", &mut self.settings.rendermode, 5);
                ui.radio_button("Render Mode: Final Normal", &mut self.settings.rendermode, 6);
                ui.radio_button("Render Mode: SSAO Map", &mut self.settings.rendermode, 7);
            });
    }

    pub fn end_frame(&mut self) {
        self.renderer.render(&mut self.imgui);
    }

    pub fn get_settings(&self) -> (f32, f32, u32, u32, f32, f32, bool) {
        (self.settings.gamma, self.settings.exposure, self.settings.environment, self.settings.rendermode, self.settings.ssao_radius, self.settings.ssao_bias, self.settings.tonemap)
    }
    pub fn get_light(&self) -> Light {
        self.settings.light1
    }
}