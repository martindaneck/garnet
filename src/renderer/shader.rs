use std::ffi::CString;
use std::fs;
use std::ptr;

pub struct Shader {
    pub id: u32,
}


fn load_source(path: &str) -> CString {
    let src = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read shader file: {}", path));

    CString::new(src)
        .expect("Shader source contains null bytes")
}

unsafe fn compile_shader(src: &CString, kind: u32, path: &str) -> u32 {
    let shader = gl::CreateShader(kind);
    gl::ShaderSource(shader, 1, &src.as_ptr(), ptr::null());
    gl::CompileShader(shader);

    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

    if success == 0 {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

        let mut buffer = vec![0u8; len as usize];
        gl::GetShaderInfoLog(
            shader,
            len,
            ptr::null_mut(),
            buffer.as_mut_ptr() as *mut _,
        );

        panic!(
            "Shader compilation failed ({}):\n{}",
            path,
            String::from_utf8_lossy(&buffer)
        );
    }

    shader
}

impl Shader {
    pub fn from_files(vertex_path: &str, fragment_path: &str) -> Shader {
        let vertex_src = load_source(vertex_path);
        let fragment_src = load_source(fragment_path);
        
        unsafe {
            let vertex_shader = compile_shader(&vertex_src, gl::VERTEX_SHADER, vertex_path);
            let fragment_shader = compile_shader(&fragment_src, gl::FRAGMENT_SHADER, fragment_path);

            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            Self { id: program }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    // uniform setters
    pub fn set_mat4(&self, name: &str, mat: &glam::Mat4) {
        let cname = CString::new(name).expect("Failed to convert to CString");
        unsafe {
            let location = gl::GetUniformLocation(self.id, cname.as_ptr());
            if location == -1 {
                eprintln!("Warning: uniform '{}' not found in shader", name);
            } else {
                gl::UniformMatrix4fv(location, 1, gl::FALSE, mat.to_cols_array().as_ptr());
            }
        }
    }

    pub fn set_vec2(&self, name: &str, vec: &glam::Vec2) {
        let cname = CString::new(name).expect("Failed to convert to CString");
        unsafe {
            let location = gl::GetUniformLocation(self.id, cname.as_ptr());
            if location == -1 {
                eprintln!("Warning: uniform '{}' not found in shader", name);
            } else {
                gl::Uniform2fv(location, 1, vec.to_array().as_ptr());
            }
        }
    }

    pub fn set_vec3(&self, name: &str, vec: &glam::Vec3) {
        let cname = CString::new(name).expect("Failed to convert to CString");
        unsafe {
            let location = gl::GetUniformLocation(self.id, cname.as_ptr());
            if location == -1 {
                eprintln!("Warning: uniform '{}' not found in shader", name);
            } else {
                gl::Uniform3fv(location, 1, vec.to_array().as_ptr());
            }
        }
    }

    pub fn set_vec3c(&self, name: &str, count: i32, values: &[glam::Vec3]) {
        let cname = CString::new(name).expect("Failed to convert to CString");
        unsafe {
            let location = gl::GetUniformLocation(self.id, cname.as_ptr());
            if location == -1 {
                eprintln!("Warning: uniform '{}' not found in shader", name);
            } else {
                gl::Uniform3fv(location, count, values.as_ptr() as *const _);
            }
        }
    }

    pub fn set_float(&self, name: &str, value: f32) {
        let cname = CString::new(name).expect("Failed to convert to CString");
        unsafe {
            let location = gl::GetUniformLocation(self.id, cname.as_ptr());
            if location == -1 {
                eprintln!("Warning: uniform '{}' not found in shader", name);
            } else {
                gl::Uniform1f(location, value);
            }
        }
    }

    pub fn set_int(&self, name: &str, value: i32) {
        let cname = CString::new(name).expect("Failed to convert to CString");
        unsafe {
            let location = gl::GetUniformLocation(self.id, cname.as_ptr());
            if location == -1 {
                eprintln!("Warning: uniform '{}' not found in shader", name);
            } else {
                gl::Uniform1i(location, value);
            }
        }
    }

    

}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}