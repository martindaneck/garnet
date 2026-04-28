use super::{Buffer, VertexArray, Texture2D, DefaultTextures};



pub struct Mesh {
    pub vao: VertexArray,
    pub vbo: Buffer,
    pub ebo: Option<Buffer>,

    pub index_count: i32,

    pub albedo: Texture2D,
    pub normal: Texture2D,
    pub orm: Texture2D,
}

impl Mesh {
    // this constructor is used by the model loader
    pub fn new(
        vertices: &[f32],
        indices: &[u32],
        albedo: Texture2D,
        normal: Texture2D,
        orm: Texture2D,
    ) -> Self {
        let vao = VertexArray::new();
        let vbo = Buffer::new();
        let ebo = Buffer::new();

        vbo.upload(vertices, gl::STATIC_DRAW);
        ebo.upload(indices, gl::STATIC_DRAW);

        // standard vertex attributes: position (3 floats), normal (3 floats), tangent + handedness (4 floats), tex coords (2 floats)
        vao.set_vertex_buffer(0, &vbo, 0, (12 * std::mem::size_of::<f32>()) as i32);
        vao.enable_attribute(0, 3, 0, 0);
        vao.enable_attribute(1, 3, 0, (3 * std::mem::size_of::<f32>()) as usize);
        vao.enable_attribute(2, 4, 0, (6 * std::mem::size_of::<f32>()) as usize);
        vao.enable_attribute(3, 2, 0, (10 * std::mem::size_of::<f32>()) as usize);
        vao.set_element_buffer(&ebo);

        Self {
            vao,
            vbo,
            ebo: Some(ebo),
            index_count: indices.len() as i32,
            albedo,
            normal,
            orm,
        }
    }


    // this constructor is used for manual mesh creation by hand (e.g., for simple shapes)
    pub fn new_manual(
        vertices: &[f32],
        indices: Option<&[u32]>,
        albedo_path: Option<&str>,
        normal_path: Option<&str>,
        orm_path: Option<&str>,
        defaults: &DefaultTextures,
    ) -> Self {
        let vao = VertexArray::new();
        let vbo = Buffer::new();

        vbo.upload(vertices, gl::STATIC_DRAW);

        // standard vertex attributes: position (3 floats), normal (3 floats), tangent + handedness (4 floats), tex coords (2 floats)
        vao.set_vertex_buffer(0, &vbo, 0, (12 * std::mem::size_of::<f32>()) as i32);
        vao.enable_attribute(0, 3, 0, 0);
        vao.enable_attribute(1, 3, 0, (3 * std::mem::size_of::<f32>()) as usize);
        vao.enable_attribute(2, 4, 0, (6 * std::mem::size_of::<f32>()) as usize);
        vao.enable_attribute(3, 2, 0, (10 * std::mem::size_of::<f32>()) as usize);

        let (ebo, index_count) = if let Some(indices) = indices {
            let ebo = Buffer::new();
            ebo.upload(indices, gl::STATIC_DRAW);
            vao.set_element_buffer(&ebo);
            (Some(ebo), indices.len() as i32)
        } else {
            (None, vertices.len() as i32 / 12) // 12 is number of components per vertex, e.g., x, y, z, nx, ny, nz, tx, ty, tz, tw, u, v
        };

        // texture setup, if paths are provided, load textures, else None
        let albedo = albedo_path
            .map(|path| Texture2D::from_file(path, "sRGB8_RGBA8"))
            .unwrap_or_else(|| defaults.white.clone());

        let normal = normal_path
            .map(|path| Texture2D::from_file(path, "Linear_RGBA8"))
            .unwrap_or_else(|| defaults.normal.clone());

        let orm = orm_path
            .map(|path| Texture2D::from_file(path, "Linear_RGBA8"))
            .unwrap_or_else(|| defaults.black.clone());

        Self {
            vao,
            vbo,
            ebo,
            index_count,
            albedo,
            normal,
            orm,
        }
    }


    pub fn triangle(
        albedo_path: Option<&str>,
        normal_path: Option<&str>,
        orm_path: Option<&str>,
    ) -> Self {
        let vertices: [f32; 36] = [
            // positions       // normals      // tangents+handedness  // tex coords
            0.0,  0.5, 0.0,   0.0, 0.0, 1.0,   1.0, 0.0, 0.0, 1.0,     0.5, 1.0,
           -0.5, -0.5, 0.0,   0.0, 0.0, 1.0,   1.0, 0.0, 0.0, 1.0,     0.0, 0.0,
            0.5, -0.5, 0.0,   0.0, 0.0, 1.0,   1.0, 0.0, 0.0, 1.0,     1.0, 0.0,
        ];

        Self::new_manual(&vertices, None, albedo_path, normal_path, orm_path, &DefaultTextures::new())
    }

    pub fn draw(&self) {
        self.vao.bind();

        self.albedo.bind(0);
        self.normal.bind(1);
        self.orm.bind(2);

        if self.ebo.is_some() {
            unsafe {
                gl::DrawElements(
                    gl::TRIANGLES,
                    self.index_count,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
            }
        } else {
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, self.index_count);
            }
        }
    }
}


pub struct FullscreenQuad {
    vao: VertexArray,
    vbo: Buffer,
}

impl FullscreenQuad {
    pub fn new() -> Self {
        let vertices: [f32; 16] = [
            // positions   // tex coords
            -1.0,  1.0,    0.0, 1.0,
            -1.0, -1.0,    0.0, 0.0,
             1.0,  1.0,    1.0, 1.0,
             1.0, -1.0,    1.0, 0.0,
        ];

        let vao = VertexArray::new();
        let vbo = Buffer::new();

        vbo.upload(&vertices, gl::STATIC_DRAW);

        vao.set_vertex_buffer(0, &vbo, 0, (4 * std::mem::size_of::<f32>()) as i32);
        vao.enable_attribute(0, 2, 0, 0);
        vao.enable_attribute(1, 2, 0, (2 * std::mem::size_of::<f32>()) as usize);

        Self { vao, vbo }
    }

    pub fn draw(&self) {
        self.vao.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }
}
    
// light cube: used for visualizing light positions
pub struct LightCube {
    vao: VertexArray,
    vbo: Buffer,
    ebo: Buffer,
    index_count: i32,
}

impl LightCube { // THIS HAS EVOLVED TO BE A GENERAL CUBE USED FOR MANY THINGS OTHER THAN LIGHT
    pub fn new() -> Self {
        let vertices: [f32; 24] = [
            // positions
            -0.1, -0.1, -0.1,
             0.1, -0.1, -0.1,
             0.1,  0.1, -0.1,
            -0.1,  0.1, -0.1,
            -0.1, -0.1,  0.1,
             0.1, -0.1,  0.1,
             0.1,  0.1,  0.1,
            -0.1,  0.1,  0.1,
        ];

        let indices: [u32; 36] = [
            // back face (-Z)
            0, 2, 1,
            0, 3, 2,

            // front face (+Z)
            4, 5, 6,
            4, 6, 7,

            // left face (-X)
            0, 7, 3,
            0, 4, 7,

            // right face (+X)
            1, 2, 6,
            1, 6, 5,

            // top face (+Y)
            3, 7, 6,
            3, 6, 2,

            // bottom face (-Y)
            0, 1, 5,
            0, 5, 4,
        ];

        let vao = VertexArray::new();
        let vbo = Buffer::new();
        let ebo = Buffer::new();

        vbo.upload(&vertices, gl::STATIC_DRAW);
        ebo.upload(&indices, gl::STATIC_DRAW);

        vao.set_vertex_buffer(0, &vbo, 0, (3 * std::mem::size_of::<f32>()) as i32);
        vao.enable_attribute(0, 3, 0, 0);
        vao.set_element_buffer(&ebo);

        Self {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
        }
    }

    pub fn draw(&self) {
        self.vao.bind();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }
}