use beryllium::*;

use anyhow::Result;
use gl33::{global_loader::*, *};
use voxel_engine::{
    draw_axis,
    engine::game::{GameResources, GameState},
    get_delta_time,
    render::{
        PolygonMode, clear_color, polygon_mode,
        renderer::{Renderer, Viewport},
        setup_3d_rendering,
        ui::UIRenderer,
    },
    render_world,
};

const VERT_SHADER_3D: &str = include_str!("../shaders/3d/vertex.glsl");
const FRAG_SHADER_3D: &str = include_str!("../shaders/3d/fragment.glsl");

const VERT_SHADER_2D: &str = include_str!("../shaders/2d/vertex.glsl");
const FRAG_SHADER_2D: &str = include_str!("../shaders/2d/fragment.glsl");

fn main() -> Result<()> {
    env_logger::init();

    let (sdl, win) = voxel_engine::init_sdl_and_win();

    unsafe { load_global_gl(&|p_name| win.get_proc_address(p_name)) };

    sdl.set_relative_mouse_mode(true).unwrap();
    win.set_swap_interval(video::GlSwapInterval::Vsync).unwrap();

    // Get actual drawable size (may differ from window size)
    let (drawable_width, drawable_height) = win.get_drawable_size();
    let viewport = Viewport {
        width: drawable_width,
        height: drawable_height,
    };

    // Set viewport to match actual drawable size
    unsafe {
        glViewport(0, 0, drawable_width, drawable_height);
    }

    let mut game = GameState::default();
    let resources = GameResources::build()?;
    let renderer = Renderer::new(
        (VERT_SHADER_3D, FRAG_SHADER_3D),
        (VERT_SHADER_2D, FRAG_SHADER_2D),
    );
    let mut ui_renderer = UIRenderer::new(&game, &resources, &viewport);

    clear_color(0.2, 0.3, 0.3, 1.0);
    polygon_mode(PolygonMode::Fill);

    // Delta time tracking
    let mut last_frame_time = sdl.get_ticks();

    'main_loop: loop {
        // Calculate delta time
        let delta_time = get_delta_time(&sdl, last_frame_time);
        last_frame_time = sdl.get_ticks();
        let fps = (1.0 / delta_time) as u32;
        game.state.fps.set_if_changed(fps);

        setup_3d_rendering();

        unsafe {
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        }

        if game.process_input_events(&sdl) {
            break 'main_loop;
        }

        game.update_player_and_world(delta_time);

        if game.world.voxels.take_dirty() {
            game.world.rebuild_mesh(&resources);
        }

        render_world(&mut game, &renderer, &viewport);

        draw_axis(&game.player.camera, &viewport);

        ui_renderer.render(&game, &resources, &renderer, &viewport);

        win.swap_window();
    }

    Ok(())
}
