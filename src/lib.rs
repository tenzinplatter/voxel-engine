use beryllium::{video::GlWindow, *};
use glam::{Vec2, Vec3};

use crate::{
    engine::game::{GameResources, GameState, vertices_from_center_and_size},
    render::{
        camera::Camera,
        debug_line::draw_debug_line,
        renderer::{Renderer, Viewport},
        vertex::Vertex2D,
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
pub fn init_sdl_and_win() -> (Sdl, GlWindow) {
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

    (sdl, win)
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

fn get_crosshair_vertices(resources: &GameResources, viewport: &Viewport) -> [Vertex2D; 6] {
    let crosshair_size = 16.0;
    let center = Vec2::new(viewport.width as f32 / 2.0, viewport.height as f32 / 2.0);

    let uvs = resources
        .atlas
        .textures
        .get("crosshair")
        .expect("Crosshair texture missing from atlas")
        .to_uvs();

    vertices_from_center_and_size(center, crosshair_size, uvs)
}

fn get_fps_vertices(resources: &GameResources, viewport: &Viewport, fps: u32) -> Vec<Vertex2D> {
    // try from to skip the '.'
    let s = format!("{fps}");
    let digits = s.chars().filter_map(|c| c.to_digit(10));
    let center = Vec2::new(viewport.width as f32 - 50.0, 50.0);
    digits
        .enumerate()
        .flat_map(|(i, d)| {
            let center = center.with_x(center.x + (i * 12) as f32);
            get_digit_vertices(resources, d, center)
        })
        .collect::<Vec<_>>()
}

fn get_digit_vertices(resources: &GameResources, digit: u32, center: Vec2) -> [Vertex2D; 6] {
    assert!(
        matches!(digit, 0..=9),
        "Should not pass a value not in 0..=9 to get_digit_vertices: {digit}"
    );

    let digit_size = 16.0;

    let uvs = resources
        .atlas
        .textures
        .get(&format!("digit_{digit}"))
        .expect("Crosshair texture missing from atlas")
        .to_uvs();

    vertices_from_center_and_size(center, digit_size, uvs)
}
