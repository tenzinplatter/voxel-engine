use bytemuck::{Pod, cast_slice};
use gl33::{GL_STATIC_DRAW, GL_TRIANGLES, global_loader::*};
use glam::Mat4;

use crate::render::{
    buffer::{Buffer, BufferType, VertexArray, buffer_data},
    shader::{ShaderProgram, ShaderUniformType},
    texture::Texture,
    vertex::Vertex,
};

pub struct Mesh {
    vao: VertexArray,
    #[allow(dead_code)]
    vbo: Buffer,
    nverticies: u32,
    pub transform: Mat4,
    pub texture: Texture,
}

impl Mesh {
    /// Creates a new mesh from vertices, transform, and texture.
    pub fn new<T, V>(vertices: &[T], transform: Mat4, texture: Texture) -> Self
    where
        T: Vertex<V> + Pod,
        V: glam_traits::FloatVec + std::ops::AddAssign,
    {
        let vao = VertexArray::new().expect("Failed to create VAO");
        vao.bind();
        let vbo = Buffer::new().expect("Failed to create VBO");
        vbo.bind(BufferType::Array);
        buffer_data(BufferType::Array, cast_slice(vertices), GL_STATIC_DRAW);
        texture.bind();
        T::configure_attributes();

        Self {
            vao,
            vbo,
            nverticies: vertices.len() as u32,
            transform,
            texture,
        }
    }

    /// Draws the mesh using the given shader program.
    pub fn draw(&self, shader_program: &ShaderProgram) {
        glBindVertexArray(self.vao.0);
        shader_program.set_uniform("model", self.transform);
        unsafe {
            glDrawArrays(GL_TRIANGLES, 0, self.nverticies as i32);
        }
    }
}
