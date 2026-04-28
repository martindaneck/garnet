pub struct Buffer {
    pub id: u32,
}

impl Buffer {
    /// create empty buffer
    pub fn new() -> Self {
        let mut id: u32 = 0;
        unsafe {
            gl::CreateBuffers(1, &mut id);
        }
        Self { id }
    }

    /// this is used for generic data like vertices
    pub fn upload<T>(&self, data: &[T], usage: u32) {
        unsafe {
            gl::NamedBufferData(
                self.id,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const _,
                usage,
            );
        }
    }

    /// this is used for stuff like UBOs - allocate and write
    pub fn allocate<T>(&self, size: usize, usage: u32) {
        unsafe {
            gl::NamedBufferData(
                self.id,
                size as isize,
                std::ptr::null(),
                usage,
            );
        }
    }
    /// write the whole buffer
    pub fn write<T>(&self, data: &[T]) {
        unsafe {
            gl::NamedBufferSubData(
                self.id,
                0,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const _,
            );
        }
    }
    // bind base for UBOs
    pub fn bind_base(&self, index: u32) {
        unsafe {
            gl::BindBufferBase(gl::UNIFORM_BUFFER, index, self.id);
        }
    }

}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

