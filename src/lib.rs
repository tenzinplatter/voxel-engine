use beryllium::{video::GlWindow, *};
use gl33::global_loader::{glViewport, load_global_gl};
use glam::Vec3;
use tracing_subscriber::EnvFilter;

use crate::{
    engine::{
        FpsTracker,
        game::{GameResources, GameState},
    },
    input::InputState,
    render::{
        PolygonMode,
        camera::Camera,
        clear_color, clear_screen,
        debug_line::draw_debug_line,
        polygon_mode,
        renderer::{Renderer, Viewport},
        setup_3d_rendering,
        ui::UIRenderer,
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

const VERT_SHADER_3D: &str = include_str!("../shaders/3d/vertex.glsl");
const FRAG_SHADER_3D: &str = include_str!("../shaders/3d/fragment.glsl");

const VERT_SHADER_2D: &str = include_str!("../shaders/2d/vertex.glsl");
const FRAG_SHADER_2D: &str = include_str!("../shaders/2d/fragment.glsl");

pub enum FrameState {
    Continue,
    Quit,
}

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
    for mesh in game.world.chunk_meshes.values() {
        renderer.render_mesh_3d(mesh, &game.player.camera, viewport);
    }
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

pub fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let (sdl, win) = setup_sdl_and_window();
    let viewport = drawable_viewport(&win);

    let mut game = GameState::default();
    let resources = GameResources::build()?;
    let renderer = Renderer::new(
        (VERT_SHADER_3D, FRAG_SHADER_3D),
        (VERT_SHADER_2D, FRAG_SHADER_2D),
    );

    let mut input_state = InputState::default();
    input_state.register_defaults();

    let mut ui_renderer = UIRenderer::new(&game, &resources, &viewport);
    let mut fps_tracker = FpsTracker::with_tracking_period(3000);
    fps_tracker.tick(sdl.get_ticks());

    while let FrameState::Continue = on_frame(
        &sdl,
        &mut fps_tracker,
        &mut game,
        &mut input_state,
        &resources,
        &renderer,
        &mut ui_renderer,
        &viewport,
        &win,
    ) {}

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn on_frame(
    sdl: &Sdl,
    fps_tracker: &mut FpsTracker,
    game: &mut GameState,
    input_state: &mut InputState,
    resources: &GameResources,
    renderer: &Renderer,
    ui: &mut UIRenderer,
    viewport: &Viewport,
    win: &GlWindow,
) -> FrameState {
    let dt = fps_tracker.tick(sdl.get_ticks()) as f32 / 1000.0;
    game.update_fps(fps_tracker.fps());

    setup_3d_rendering();
    clear_screen();

    input_state.poll(sdl);
    if input_state.should_quit() {
        return FrameState::Quit;
    }

    game.on_frame(resources, dt, input_state);
    game.world.rebuild_dirty_chunks(resources);

    game.world.voxels.end_frame();
    input_state.end_frame();

    render_world(game, renderer, viewport);
    draw_axis(&game.player.camera, viewport);
    ui.render(game, resources, renderer, viewport);
    win.swap_window();

    FrameState::Continue
}
