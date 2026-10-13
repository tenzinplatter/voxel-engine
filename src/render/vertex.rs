use bytemuck::{Pod, Zeroable};
use gl33::{global_loader::*, *};
use glam::{Vec2, Vec3};

pub(crate) trait Vertex {
    /// Configures OpenGL vertex attributes for this vertex type.
    fn configure_attributes();
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct VertexTex {
    position: Vec3,
    tex: Vec2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(crate) struct Vertex2D {
    pub(crate) position: Vec2,
    pub(crate) tex: Vec2,
}

impl Vertex for VertexTex {
    fn configure_attributes() {
        unsafe {
            let vec3_size: i32 = size_of::<Vec3>().try_into().unwrap();
            let vec2_size: i32 = size_of::<Vec2>().try_into().unwrap();
            let stride = vec3_size + vec2_size;
            // setup position attribute
            glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE.0 as u8, stride, std::ptr::null());
            // setup color attribute
            glVertexAttribPointer(
                1,
                2,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                stride,
                // texture coords are offset from beginning by the size of the vec3 with the point
                vec3_size as *const _,
            );

            glEnableVertexAttribArray(0);
            glEnableVertexAttribArray(1);
        }
    }
}

impl VertexTex {
    /// Creates a new textured vertex at the given position with texture coordinates.
    pub(crate) fn new(point: Vec3, tex: Vec2) -> Self {
        Self {
            position: point,
            tex,
        }
    }
}

impl Vertex for Vertex2D {
    fn configure_attributes() {
        unsafe {
            let vec2_size: i32 = size_of::<Vec2>() as i32;

            // setup position attribute
            glVertexAttribPointer(
                0,
                2,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                2 * vec2_size,
                std::ptr::null(),
            );

            // setup texture attribute
            glVertexAttribPointer(
                1,
                2,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                2 * vec2_size,
                vec2_size as *const _,
            );

            glEnableVertexAttribArray(0);
            glEnableVertexAttribArray(1);
        }
    }
}
