pub struct VertexArray {
    pub id: u32,
}

impl VertexArray {
    pub fn new() -> Self {
        let mut id: u32 = 0;
        unsafe {
            gl::CreateVertexArrays(1, &mut id);
        }
        Self { id }
    }

    /// bind vbo to vao at specified index
    pub fn set_vertex_buffer(
        &self,
        binding_index: u32,
        buffer: &super::buffer::Buffer,
        offset: usize,
        stride: i32,
    ) {
        unsafe {
            gl::VertexArrayVertexBuffer(
                self.id,
                binding_index,
                buffer.id,
                offset as isize,
                stride,
            );  
        }
    }

    pub fn set_element_buffer(&self, buffer: &super::buffer::Buffer) {
        unsafe {
            gl::VertexArrayElementBuffer(self.id, buffer.id);
        }
    }

    pub fn enable_attribute(&self, attrib_index: u32, size: i32, binding_index: u32, offset: usize) {
        unsafe {
            gl::EnableVertexArrayAttrib(self.id, attrib_index);
            gl::VertexArrayAttribFormat(
                self.id,
                attrib_index,
                size,
                gl::FLOAT,
                gl::FALSE,
                offset as u32,
            );
            gl::VertexArrayAttribBinding(self.id, attrib_index, binding_index);
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

