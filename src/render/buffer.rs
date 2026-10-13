use gl33::{global_loader::*, *};

/// Basic wrapper for a [Vertex Array Object]
/// (https://www.khronos.org/opengl/wiki/Vertex_Specification#Vertex_Array_Object).
pub(crate) struct VertexArray(pub(crate) u32);

impl VertexArray {
    /// Creates a new vertex array object.
    pub(crate) fn new() -> Option<Self> {
        let mut vao = 0;
        unsafe { glGenVertexArrays(1, &mut vao) };
        if vao != 0 { Some(Self(vao)) } else { None }
    }

    /// Binds this vertex array as the current vertex array object.
    pub(crate) fn bind(&self) {
        glBindVertexArray(self.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BufferType {
    Array = GL_ARRAY_BUFFER.0 as isize,
}
/// Basic wrapper for a [Buffer
/// Object](https://www.khronos.org/opengl/wiki/Buffer_Object).
pub(crate) struct Buffer(u32);

impl Buffer {
    /// Creates a new vertex buffer.
    pub(crate) fn new() -> Option<Self> {
        let mut vbo = 0;
        unsafe {
            glGenBuffers(1, &mut vbo);
        }
        if vbo != 0 { Some(Self(vbo)) } else { None }
    }

    /// Binds this vertex buffer for the given type.
    pub(crate) fn bind(&self, ty: BufferType) {
        unsafe { glBindBuffer(GLenum(ty as u32), self.0) }
    }
}

/// Places a slice of data into a previously-bound buffer.
pub(crate) fn buffer_data(ty: BufferType, data: &[u8], usage: GLenum) {
    unsafe {
        glBufferData(
            GLenum(ty as u32),
            size_of_val(data) as isize,
            data.as_ptr().cast(),
            usage,
        );
    }
}
