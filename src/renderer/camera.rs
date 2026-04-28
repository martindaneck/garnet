use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub front: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub world_up: Vec3,

    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
}

impl Camera {
    pub fn new(position: Vec3, up: Vec3, yaw: f32, pitch: f32) -> Self {
        let mut camera = Self {
            position,
            front: Vec3::new(0.0, 0.0, -1.0),
            up,
            right: Vec3::ZERO,
            world_up: up,
            yaw,
            pitch,
            fov: 90.0,
        };
        camera.update_camera_vectors();
        camera
    }

    fn update_camera_vectors(&mut self) {
        // Convert yaw and pitch to front vector
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        self.front = Vec3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        )
        .normalize();
        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.front, self.up)
    }

    pub fn get_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov.to_radians(), aspect_ratio, 0.1, 100.0)
    }

    // movement
    pub fn process_keyboard(&mut self, direction: CameraMovement, delta_time: f32) {
        let velocity = 3.0 * delta_time;
        let mut front = self.front;
        front.y = 0.0; // Lock y movement
        front = front.normalize();
        let mut right = self.right;
        right.y = 0.0; // Lock y movement
        right = right.normalize();
        match direction {
            CameraMovement::Forward => self.position += front * velocity,
            CameraMovement::Backward => self.position -= front * velocity,
            CameraMovement::Left => self.position -= right * velocity,
            CameraMovement::Right => self.position += right * velocity,
            CameraMovement::Up => self.position += self.world_up * velocity,
            CameraMovement::Down => self.position -= self.world_up * velocity,
        }
    }

    pub fn process_mouse_movement(&mut self, xoffset: f32, yoffset: f32, sensitivity: f32) {
        let xoffset = xoffset * sensitivity;
        let yoffset = yoffset * sensitivity;
        self.yaw += xoffset;
        self.pitch += yoffset;
        // Constrain pitch
        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }
        self.update_camera_vectors();
    }
}

pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}