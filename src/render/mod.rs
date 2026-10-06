pub mod atlas;
pub mod buffer;
pub mod camera;
pub mod debug_line;
pub mod mesh;
pub mod renderer;
pub mod shader;
pub mod texture;
pub mod ui;
pub mod vertex;

use gl33::{global_loader::*, *};

/// The polygon display modes you can set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonMode {
    /// Just show the points.
    Point = GL_POINT.0 as isize,
    /// Just show the lines.
    Line = GL_LINE.0 as isize,
    /// Fill in the polygons.
    Fill = GL_FILL.0 as isize,
}

/// Sets the font and back polygon mode to the mode given.
pub fn polygon_mode(mode: PolygonMode) {
    unsafe { glPolygonMode(GL_FRONT_AND_BACK, GLenum(mode as u32)) };
}

/// Sets the clear color for the framebuffer.
pub fn clear_color(r: f32, g: f32, b: f32, a: f32) {
    unsafe {
        glClearColor(r, g, b, a);
    }
}

pub fn setup_3d_rendering() {
    unsafe {
        glEnable(GL_DEPTH_TEST);
        glDisable(GL_BLEND);
    }
}

pub fn setup_2d_rendering() {
    unsafe {
        glDisable(GL_DEPTH_TEST);
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }
}

pub fn clear_screen() {
    unsafe {
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    }
}
