use glam::Mat4;

use crate::{
    degrees_to_radians,
    render::{
        camera::Camera,
        mesh::Mesh,
        shader::{ShaderProgram, ShaderUniformType},
    },
};

pub(crate) struct Renderer {
    shader_program_3d: ShaderProgram,
    shader_program_2d: ShaderProgram,
}

// TEMP: should this live here? should it exist?
pub(crate) struct Viewport {
    pub(crate) width: i32,
    pub(crate) height: i32,
}

impl Renderer {
    /// Creates a new renderer with the given vertex and fragment shaders.
    /// # Arguments
    /// * `shaders_3d` - A tuple containing the vertex and fragment shader source code for 3D rendering.
    ///   Order: (vertex_shader, fragment_shader)
    /// * `shaders_2d` - A tuple containing the vertex and fragment shader source code for 2D rendering.
    ///   Order: (vertex_shader, fragment_shader)
    pub(crate) fn new(shaders_3d: (&str, &str), shaders_2d: (&str, &str)) -> Self {
        let shader_program_3d = ShaderProgram::from_vert_frag(shaders_3d.0, shaders_3d.1)
            .expect("Failed to create shader program");
        let shader_program_2d = ShaderProgram::from_vert_frag(shaders_2d.0, shaders_2d.1)
            .expect("Failed to create shader program");

        Self {
            shader_program_3d,
            shader_program_2d,
        }
    }

    /// Binds the renderer's shader program for use.
    fn bind_3d(&self) {
        self.shader_program_3d.use_program()
    }

    fn bind_2d(&self) {
        self.shader_program_2d.use_program()
    }

    /// Renders a mesh with the given camera and viewport settings.
    pub(crate) fn render_mesh_3d(&self, mesh: &Mesh, camera: &Camera, viewport: &Viewport) {
        self.bind_3d();
        let view = camera.view_matrix();
        let proj = Mat4::perspective_rh_gl(
            degrees_to_radians(45.0),
            viewport.width as f32 / viewport.height as f32,
            0.1,
            100.0,
        );

        self.shader_program_3d.set_uniform("view", view);
        self.shader_program_3d.set_uniform("proj", proj);
        self.shader_program_3d
            .set_uniform("transform", mesh.transform);

        mesh.draw(&self.shader_program_3d);
    }

    pub(crate) fn render_mesh_2d(&self, mesh: &Mesh, viewport: &Viewport) {
        self.bind_2d();
        let proj = Mat4::orthographic_rh_gl(
            0.0,
            viewport.width as f32,
            viewport.height as f32,
            0.0,
            -1.0,
            1.0,
        );
        self.shader_program_2d.set_uniform("proj", proj);

        mesh.draw(&self.shader_program_2d);
    }
}
