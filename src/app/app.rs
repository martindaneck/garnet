use std::time::{Duration, Instant};
use glfw::{Action, Context, Key};
use glam::{Mat4, Vec3};

use crate::renderer::{Camera, CameraMovement, Shader};

pub struct App {
    glfw: glfw::Glfw,
    pub window: glfw::PWindow,
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,

    pub width: i32,
    pub height: i32,
    resized: bool,

    fps_dts: Vec<f64>,
    fps: u32,

    camera: Camera,
    last_mouse_x: f64,
    last_mouse_y: f64,
    first_mouse: bool,
    last_frame_time: f64,
    dt: f64,

    mouse_locked: bool,
    pub show_imgui: bool,
}

impl App {
    // Create a new App instance
    pub fn new(width: i32, height: i32, title: &str) -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

        glfw.window_hint(glfw::WindowHint::ContextRobustness(glfw::ContextRobustnessHint::NoRobustness));
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        let (mut window, events) = glfw
            .create_window(width.try_into().unwrap(), height.try_into().unwrap(), title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");

        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        // cursor
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_cursor_pos_polling(true);

        gl::load_with(|s| {
            window.get_proc_address(s)
                .map(|f| f as *const _)
                .unwrap_or(std::ptr::null())
        });

        let (width, height) = window.get_framebuffer_size();

        // initial opengl state setup
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            // enable depth testing
            gl::Enable(gl::DEPTH_TEST);
            // cull faces
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);
        }

        let last_frame_time = glfw.get_time();

        App { glfw, window, events, width, height, resized: false, fps_dts: Vec::new(), fps: 240, //initial value to show (arbitrary)
            camera: Camera::new(glam::Vec3::new(0.0, 0.0, 3.0), glam::Vec3::Y, -90.0, 0.0),
            last_mouse_x: (width / 2) as f64,
            last_mouse_y: (height / 2) as f64,
            first_mouse: true,
            last_frame_time,
            dt: 0.0,
            mouse_locked: true,
            show_imgui: true,
        }
    }

    pub fn is_running(&self) -> bool {
        !self.window.should_close()
    }

    pub fn begin_frame(&mut self) {
        // delta time
        let current_frame_time = self.glfw.get_time();
        let delta_time = current_frame_time - self.last_frame_time;
        self.last_frame_time = current_frame_time;
        self.dt = delta_time;

        // handle fps
        self.update_fps();
        
        // poll events
        self.glfw.poll_events();

        // handle events
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::FramebufferSize(w, h) => {
                    self.width = w;
                    self.height = h;
                    self.resized = true;

                    unsafe {
                        gl::Viewport(0, 0, w, h);
                    }
                }

                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true);
                }

                glfw::WindowEvent::Key(Key::E, _, Action::Press, _) => {
                    self.mouse_locked = !self.mouse_locked;
                    if self.mouse_locked {
                        self.window.set_cursor_mode(glfw::CursorMode::Disabled);
                    } else {
                        self.window.set_cursor_mode(glfw::CursorMode::Normal);
                    };
                    self.window.set_cursor_mode(if self.mouse_locked { glfw::CursorMode::Disabled } else { glfw::CursorMode::Normal });
                    if !self.mouse_locked {
                        self.first_mouse = true;
                    }
                }

                glfw::WindowEvent::Key(Key::F3, _, Action::Press, _) => {
                    self.show_imgui = !self.show_imgui;
                }

                _ => {}
            }
        }

        // keyboard input
        let mut speed: f32 = 1.0;
        if self.window.get_key(Key::LeftShift) == Action::Press { speed *= 2.0; }
        if self.window.get_key(Key::W) == Action::Press { self.camera.process_keyboard(CameraMovement::Forward, delta_time as f32 * speed); }
        if self.window.get_key(Key::S) == Action::Press { self.camera.process_keyboard(CameraMovement::Backward, delta_time as f32 * speed); }
        if self.window.get_key(Key::A) == Action::Press { self.camera.process_keyboard(CameraMovement::Left, delta_time as f32 * speed); }
        if self.window.get_key(Key::D) == Action::Press { self.camera.process_keyboard(CameraMovement::Right, delta_time as f32 * speed); }
        if self.window.get_key(Key::Space) == Action::Press { self.camera.process_keyboard(CameraMovement::Up, delta_time as f32 * speed); }
        if self.window.get_key(Key::LeftControl) == Action::Press { self.camera.process_keyboard(CameraMovement::Down, delta_time as f32); }

        // mouse input
        if self.mouse_locked {
            let (mouse_x, mouse_y) = self.window.get_cursor_pos();
            if self.first_mouse {
                self.last_mouse_x = mouse_x;
                self.last_mouse_y = mouse_y;
                self.first_mouse = false;
            }
            let dx = (mouse_x - self.last_mouse_x) as f32;
            let dy = (self.last_mouse_y - mouse_y) as f32;
            self.last_mouse_x = mouse_x;
            self.last_mouse_y = mouse_y;
            self.camera.process_mouse_movement(dx, dy, 0.2); 
        }
          

        // openGL stuff: clear screen
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn end_frame(&mut self) {
        self.window.swap_buffers();
    }

    pub fn framebuffer_size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    pub fn was_resized(&mut self) -> bool {
        self.resized
    }

    pub fn clear_resized_flag(&mut self) {
        self.resized = false;
    }

    pub fn get_view_projection_position(&self) -> (Mat4, Mat4, Vec3) {
        let aspect_ratio = self.width as f32 / self.height as f32;
        let view = self.camera.get_view_matrix();
        let projection = self.camera.get_projection_matrix(aspect_ratio);
        let camera_position = self.camera.position;

        // return both matrices and position
        (view, projection, camera_position)
    }

    pub fn update_fps(&mut self) {
        self.fps_dts.push(self.dt);
        let total_time: f64 = self.fps_dts.iter().sum();
        if total_time >= 2.0 {
            let frames = self.fps_dts.len();
            self.fps = (frames as f64 / total_time).round() as u32;
            self.fps_dts.clear();
        }
    }

    pub fn get_fps(&self) -> u32 {
        self.fps
    }

}

