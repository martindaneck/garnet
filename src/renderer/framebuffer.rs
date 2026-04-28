use super::{Texture2D};

pub struct Framebuffer {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub color: Vec<Texture2D>,
    pub depth: Option<Texture2D>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let mut id = 0;
        unsafe {
            gl::CreateFramebuffers(1, &mut id);
        }

        Self {
            id,
            width,
            height,
            color: Vec::new(),
            depth: None,
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn add_color_attachment(&mut self, texture: Texture2D) {
        let index = self.color.len() as u32;
        unsafe {
            gl::NamedFramebufferTexture(self.id, gl::COLOR_ATTACHMENT0 + index, texture.id, 0);
        }
        self.color.push(texture);

        let attachments: Vec<u32> = (0..self.color.len())
            .map(|i| gl::COLOR_ATTACHMENT0 + i as u32)
            .collect();
        unsafe {
            gl::NamedFramebufferDrawBuffers(
                self.id,
                attachments.len() as i32,
                attachments.as_ptr(),
            );
        }
    }

    pub fn set_depth_attachment(&mut self, texture: Texture2D) {
        unsafe {
            gl::NamedFramebufferTexture(self.id, gl::DEPTH_ATTACHMENT, texture.id, 0);
        }
        self.depth = Some(texture);
    }

    pub fn check_complete(&self) {
        let status = unsafe {
            gl::CheckNamedFramebufferStatus(self.id, gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE
        };
        assert!(status, "Framebuffer is not complete!");
    }

    pub fn recreate(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        // recreate attachments
        for (i, tex) in self.color.iter_mut().enumerate() {
            tex.recreate(width, height);

            unsafe {
                gl::NamedFramebufferTexture(
                    self.id,
                    gl::COLOR_ATTACHMENT0 + i as u32,
                    tex.id,
                    0,
                );
            }
        }
        if let Some(depth_tex) = &mut self.depth {
            depth_tex.recreate(width, height);
            unsafe {
                gl::NamedFramebufferTexture(
                    self.id,
                    gl::DEPTH_ATTACHMENT,
                    depth_tex.id,
                    0,
                );
            }
        }
    }
}

