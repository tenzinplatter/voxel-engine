use anyhow::Result;
use tracing_subscriber::EnvFilter;
use voxel_engine::{
    draw_axis,
    engine::{
        FpsTracker,
        game::{GameResources, GameState},
    },
    input::InputState,
    render::{clear_screen, renderer::Renderer, setup_3d_rendering, ui::UIRenderer},
    render_world,
};

const VERT_SHADER_3D: &str = include_str!("../shaders/3d/vertex.glsl");
const FRAG_SHADER_3D: &str = include_str!("../shaders/3d/fragment.glsl");

const VERT_SHADER_2D: &str = include_str!("../shaders/2d/vertex.glsl");
const FRAG_SHADER_2D: &str = include_str!("../shaders/2d/fragment.glsl");

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let (sdl, win) = voxel_engine::setup_sdl_and_window();
    let viewport = voxel_engine::drawable_viewport(&win);

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

    'main_loop: loop {
        let dt = fps_tracker.tick(sdl.get_ticks()) as f32 / 1000.0;
        game.update_fps(fps_tracker.fps());

        setup_3d_rendering();
        clear_screen();

        input_state.poll(&sdl);
        if input_state.should_quit() {
            break 'main_loop;
        }

        game.on_frame(&resources, dt, &input_state);
        game.world.rebuild_dirty_chunks(&resources);

        game.world.voxels.end_frame();
        input_state.end_frame();

        render_world(&mut game, &renderer, &viewport);
        draw_axis(&game.player.camera, &viewport);
        ui_renderer.render(&game, &resources, &renderer, &viewport);
        win.swap_window();
    }

    Ok(())
}
