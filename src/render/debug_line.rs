use gl33::{GL_ARRAY_BUFFER, GL_DYNAMIC_DRAW, GL_FALSE, GL_FLOAT, GL_LINES, global_loader::*};
use glam::{Mat4, Vec3};
use std::mem::size_of;
use std::sync::LazyLock;

use crate::render::{
    camera::Camera,
    renderer::Viewport,
    shader::{ShaderProgram, ShaderUniformType},
};

static DEBUG_LINE_SHADER: LazyLock<ShaderProgram> = LazyLock::new(|| {
    let vert = include_str!("../../shaders/debug/vertex.glsl");
    let frag = include_str!("../../shaders/debug/fragment.glsl");
    ShaderProgram::from_vert_frag(vert, frag).expect("Failed to compile debug line shader")
});

pub(crate) fn draw_debug_line(
    start: Vec3,
    end: Vec3,
    color: Vec3,
    camera: &Camera,
    viewport: &Viewport,
) {
    unsafe {
        let shader = &*DEBUG_LINE_SHADER;
        shader.use_program();

        // Vertex data: just positions
        let vertices: [f32; 6] = [start.x, start.y, start.z, end.x, end.y, end.z];

        // Create VAO/VBO
        let mut vao = 0;
        let mut vbo = 0;
        glGenVertexArrays(1, &mut vao);
        glGenBuffers(1, &mut vbo);

        glBindVertexArray(vao);
        glBindBuffer(GL_ARRAY_BUFFER, vbo);
        glBufferData(
            GL_ARRAY_BUFFER,
            size_of::<[f32; 6]>() as isize,
            vertices.as_ptr() as *const _,
            GL_DYNAMIC_DRAW,
        );

        // Position attribute (location = 0)
        glVertexAttribPointer(
            0,
            3,
            GL_FLOAT,
            GL_FALSE.0 as u8,
            (3 * size_of::<f32>()) as i32,
            std::ptr::null(),
        );
        glEnableVertexAttribArray(0);

        // Set uniforms
        let view = camera.view_matrix();
        let proj = Mat4::perspective_rh_gl(
            crate::degrees_to_radians(45.0),
            viewport.width as f32 / viewport.height as f32,
            0.1,
            100.0,
        );

        shader.set_uniform("view", view);
        shader.set_uniform("proj", proj);
        shader.set_uniform("line_color", color);

        // Check for OpenGL errors
        let error = glGetError();
        if error != gl33::GL_NO_ERROR {
            println!("OpenGL error after setting uniforms: {:?}", error);
        }

        // Optional: Make line thicker (default is 1.0)
        glLineWidth(3.0);

        // Draw the line
        glDrawArrays(GL_LINES, 0, 2);

        // Cleanup
        glBindVertexArray(0); // Unbind VAO
        glDeleteVertexArrays(1, &vao);
        glDeleteBuffers(1, &vbo);
    }
}
