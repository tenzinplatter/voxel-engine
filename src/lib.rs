use beryllium::{video::GlWindow, *};
use gl33::global_loader::{glViewport, load_global_gl};
use glam::Vec3;

use crate::{
    engine::game::GameState,
    render::{
        PolygonMode,
        camera::Camera,
        clear_color,
        debug_line::draw_debug_line,
        polygon_mode,
        renderer::{Renderer, Viewport},
    },
};

pub mod engine;
pub mod input;
pub mod physics;
pub mod player;
pub mod render;
pub mod utils;

const WIDTH: i32 = 1600;
const HEIGHT: i32 = 900;

// make window float with my niri setup
const WINDOW_TITLE: &str = "(float)";

/// Initializes SDL and creates an OpenGL window with default settings.
pub fn setup_sdl_and_window() -> (Sdl, GlWindow) {
    let sdl = Sdl::init(init::InitFlags::EVERYTHING);

    sdl.set_gl_context_major_version(3).unwrap();
    sdl.set_gl_context_minor_version(3).unwrap();
    sdl.set_gl_profile(video::GlProfile::Core).unwrap();

    let win_args = video::CreateWinArgs {
        title: WINDOW_TITLE,
        width: WIDTH,
        height: HEIGHT,
        allow_high_dpi: false,
        borderless: false,
        resizable: false,
    };

    let win = sdl
        .create_gl_window(win_args)
        .expect("Failed to create window");
    win.set_swap_interval(video::GlSwapInterval::Immediate)
        .unwrap();

    unsafe { load_global_gl(&|p_name| win.get_proc_address(p_name)) };

    sdl.set_relative_mouse_mode(true).unwrap();
    win.set_swap_interval(video::GlSwapInterval::Vsync).unwrap();

    clear_color(0.2, 0.3, 0.3, 1.0);
    polygon_mode(PolygonMode::Fill);

    (sdl, win)
}

pub fn drawable_viewport(win: &GlWindow) -> Viewport {
    let (drawable_width, drawable_height) = win.get_drawable_size();
    unsafe {
        glViewport(0, 0, drawable_width, drawable_height);
    }
    Viewport {
        width: drawable_width,
        height: drawable_height,
    }
}

/// Converts degrees to radians.
pub fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}

/// Converts radians to degrees.
pub fn radians_to_degrees(radians: f32) -> f32 {
    radians * 180.0 / std::f32::consts::PI
}

/// Returns the delta time since the last frame in seconds.
pub fn get_delta_time(sdl: &Sdl, last_frame_time: u32) -> f32 {
    let current_frame_time = sdl.get_ticks();
    let delta_time = current_frame_time - last_frame_time;

    delta_time as f32 / 1000.0
}

pub fn render_world(game: &mut GameState, renderer: &Renderer, viewport: &Viewport) {
    renderer.render_mesh_3d(
        game.world
            .mesh
            .as_ref()
            .expect("Mesh shouldve been build on world init"),
        &game.player.camera,
        viewport,
    );
}

pub fn draw_axis(camera: &Camera, viewport: &Viewport) {
    let axis_length = 10.0;
    // draw at y=1 to be above ground
    let origin = Vec3::ZERO.with_y(1.);

    // X axis in red
    draw_debug_line(
        origin,
        origin + Vec3::new(axis_length, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        camera,
        viewport,
    );

    // Y axis in green
    draw_debug_line(
        origin,
        origin + Vec3::new(0.0, axis_length, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        camera,
        viewport,
    );

    // Z axis in blue
    draw_debug_line(
        origin,
        origin + Vec3::new(0.0, 0.0, axis_length),
        Vec3::new(0.0, 0.0, 1.0),
        camera,
        viewport,
    );
}
