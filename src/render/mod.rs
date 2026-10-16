pub(crate) mod atlas;
mod buffer;
pub(crate) mod camera;
pub(crate) mod debug_line;
pub(crate) mod mesh;
pub(crate) mod renderer;
mod shader;
mod texture;
pub(crate) mod ui;
pub(crate) mod vertex;

use gl33::{global_loader::*, *};

/// Sets the clear color for the framebuffer.
pub(crate) fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe {
        glClearColor(r, g, b, a);
    }
}

pub(crate) fn setup_3d_rendering() {
    unsafe {
        glEnable(GL_DEPTH_TEST);
        
        glEnable(GL_CULL_FACE);
        glCullFace(GL_BACK);
        glFrontFace(GL_CCW);

        glDisable(GL_BLEND);
    }
}

fn setup_2d_rendering() {
    unsafe {
        glDisable(GL_DEPTH_TEST);
        glDisable(GL_CULL_FACE);

        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }
}

pub(crate) fn clear_screen() {
    unsafe {
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    }
}
