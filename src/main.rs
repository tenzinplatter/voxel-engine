use beryllium::*;

use anyhow::Result;
use gl33::{global_loader::*, *};
use voxel_engine::{
    draw_axis,
    engine::{
        FpsTracker,
        game::{GameResources, GameState},
    },
    get_delta_time,
    render::{
        PolygonMode, clear_color, clear_screen, polygon_mode,
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

    let (sdl, win) = voxel_engine::setup_sdl_and_window();
    let viewport = voxel_engine::drawable_viewport(&win);

    let mut game = GameState::default();
    let resources = GameResources::build()?;
    let renderer = Renderer::new(
        (VERT_SHADER_3D, FRAG_SHADER_3D),
        (VERT_SHADER_2D, FRAG_SHADER_2D),
    );

    let mut ui_renderer = UIRenderer::new(&game, &resources, &viewport);
    let mut fps_tracker = FpsTracker::new(3000);
    fps_tracker.tick(sdl.get_ticks());

    'main_loop: loop {
        let dt = fps_tracker.tick(sdl.get_ticks()) as f32 / 1000.0;
        game.update_fps(fps_tracker.fps());

        setup_3d_rendering();
        clear_screen();

        if game.process_input_events(&sdl) {
            break 'main_loop;
        }

        game.update_player_and_world(dt);

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
