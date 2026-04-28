use gltf::json::path;
use glam::{Mat4, Vec3};

use super::{Shader, Texture2D, Framebuffer, LightCube};

#[derive(Clone)]
pub struct TextureCube {
    pub id: u32,
    pub size: u32,
    pub internal_format: u32,
}

impl TextureCube {
    pub fn empty(size: u32, internal_format: u32, mipmapped: bool) -> Self {
        let mut id = 0;

        unsafe {
            gl::CreateTextures(gl::TEXTURE_CUBE_MAP, 1, &mut id);

            let levels = if mipmapped {
                (size as f32).log2().floor() as i32 + 1
            } else {
                1
            };

            gl::TextureStorage2D(
                id,
                levels,
                internal_format,
                size as i32,
                size as i32,
            );

            gl::TextureParameteri(id, gl::TEXTURE_MIN_FILTER, if mipmapped { gl::LINEAR_MIPMAP_LINEAR as i32 } else { gl::LINEAR as i32 });
            gl::TextureParameteri(id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(id, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TextureParameteri(id, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TextureParameteri(id, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);

            if mipmapped {
                gl::GenerateTextureMipmap(id);
            }

            Self {
                id,
                size,
                internal_format,
            }
        }
    }

    pub fn bind(&self, unit: u32) {
        unsafe {
            gl::BindTextureUnit(unit, self.id);
        }
    }

    pub fn from_hdr_equirectangular(path: &str, size: u32) -> Self {
        // Load the HDR equirectangular image
        let hdr_texture = Texture2D::from_hdr_file(path);
        // Create the cubemap
        let cubemap = TextureCube::empty(size, gl::RGB16F, true);
        // shader
        let shader = Shader::from_files(
            "assets/shaders/cubemap.vertex.glsl",
            "assets/shaders/equirectangulartocubemap.fragment.glsl",
        );
        // cube
        let cube = LightCube::new();
        // temporary texture
        let mut temp_texture = Texture2D::empty(size, size, gl::RGB16F, gl::NEAREST, gl::CLAMP_TO_EDGE);
        // Create the framebuffer
        let mut framebuffer = Framebuffer::new(size, size);
        framebuffer.add_color_attachment(temp_texture.clone());
        framebuffer.check_complete();

        // capture matrices
        let projection = Mat4::perspective_rh_gl(90.0f32.to_radians(), 1.0, 0.01, 10.0);
        let views = [
            Mat4::look_at_rh(Vec3::ZERO, Vec3::X, -Vec3::Y),
            Mat4::look_at_rh(Vec3::ZERO, -Vec3::X, -Vec3::Y),
            Mat4::look_at_rh(Vec3::ZERO, Vec3::Y, Vec3::Z),
            Mat4::look_at_rh(Vec3::ZERO, -Vec3::Y, -Vec3::Z),
            Mat4::look_at_rh(Vec3::ZERO, Vec3::Z, -Vec3::Y),
            Mat4::look_at_rh(Vec3::ZERO, -Vec3::Z, -Vec3::Y),
        ];

        // render
        shader.bind(); // bind the shader
        shader.set_int("equirectangularMap", 0); // set the equirectangular map
        shader.set_mat4("projection", &projection); // set the projection matrix
        hdr_texture.bind(0); // bind the HDR texture
        framebuffer.bind(); // bind the framebuffer
        unsafe {  // set viewport and disable culling and depth test, clear screen
            gl::Viewport(0, 0, size as i32, size as i32);
            gl::Disable(gl::CULL_FACE);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }
        for i in 0..6 { // the six faces
            shader.set_mat4("view", &views[i]); // set the view matrix
            unsafe {
                // clear temporary texture
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                // draw the cube
                cube.draw();

                // copy temporary texture to cubemap face
                gl::CopyImageSubData( // NOTES TO SELF: IT IS ABSOLUTELY NECESSARY TO DRAW TO TEMPORARY TEXTURES FIRST AND COPY OVER TO THE CUBEMAP LATER,
                                      // OTHERWISE THE GRAPHICS DRIVERS THEMSELVES WILL PULL A DIPSHIT ON YOU AND CLEAR YOUR LAST FACE WITH GRAY
                    temp_texture.id,
                    gl::TEXTURE_2D,
                    0,
                    0,
                    0,
                    0,
                    cubemap.id,
                    gl::TEXTURE_CUBE_MAP,
                    0,
                    0,
                    0,
                    i as i32,
                    size as i32,
                    size as i32,
                    1,
                );
            }
        }
        unsafe { // re-enable culling and depth test
            gl::Enable(gl::CULL_FACE);
            // generate mipmaps
            gl::GenerateTextureMipmap(cubemap.id);
        }
        // Create a new TextureCube
        cubemap
    }

    pub fn delete(&self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }

    fn load_dds_raw(path: &str) -> (u32, u32, u32, u32, u32, Vec<u8>) {
        use std::{fs::File, io::Read};

        let mut file = File::open(path).expect("Failed to open DDS");

        let mut header = [0u8; 128];
        file.read_exact(&mut header).unwrap();

        assert!(&header[0..4] == b"DDS ");

        let height = u32::from_le_bytes(header[12..16].try_into().unwrap());
        let width  = u32::from_le_bytes(header[16..20].try_into().unwrap());

        let fourcc = &header[84..88];
        assert!(fourcc == b"DX10", "Only DX10 DDS supported");

        // Read DX10 header
        let mut dx10 = [0u8; 20];
        file.read_exact(&mut dx10).unwrap();

        let dxgi_format = u32::from_le_bytes(dx10[0..4].try_into().unwrap());

        let (internal, format, ty, bytes_per_pixel) = match dxgi_format {
            10 => (gl::RGBA16F, gl::RGBA, gl::HALF_FLOAT, 8),   // DXGI_FORMAT_R16G16B16A16_FLOAT
            2  => (gl::RGBA32F, gl::RGBA, gl::FLOAT, 16),      // DXGI_FORMAT_R32G32B32A32_FLOAT
            _ => panic!("Unsupported DXGI format {}", dxgi_format),
        };

        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        let expected = (width * height) as usize * bytes_per_pixel;
        assert!(
            data.len() == expected,
            "DDS size mismatch: got {}, expected {}",
            data.len(),
            expected
        );

        (width, height, internal, format, ty, data)
    }

    pub fn from_dds(path: &str, mip_levels: u32) -> Self {
        let faces = ["px", "nx", "py", "ny", "pz", "nz"];

        // Load base mip to get size + format
        let (size, _, internal_format, _, _, _) =
            Self::load_dds_raw(&format!("{}/m0_px.dds", path));

        let mut id = 0;
        unsafe {
            gl::CreateTextures(gl::TEXTURE_CUBE_MAP, 1, &mut id);
            gl::TextureStorage2D(
                id,
                mip_levels as i32,
                internal_format,
                size as i32,
                size as i32,
            );
        }

        for mip in 0..mip_levels {
            for (face_idx, face) in faces.iter().enumerate() {
                let (w, h, _, format, ty, data) =
                    Self::load_dds_raw(&format!("{}/m{}_{}.dds", path, mip, face));

                unsafe {
                    gl::TextureSubImage3D(
                        id,
                        mip as i32,
                        0,
                        0,
                        face_idx as i32,
                        w as i32,
                        h as i32,
                        1,
                        format,
                        ty,
                        data.as_ptr() as *const _,
                    );
                }
            }
        }

        unsafe {
            gl::TextureParameteri(id, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
            gl::TextureParameteri(id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(id, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TextureParameteri(id, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TextureParameteri(id, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
        }

        Self {
            id,
            size,
            internal_format,
        }
    }
}
