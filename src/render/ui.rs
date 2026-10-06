use glam::{Mat4, Vec2};

use crate::{
    engine::game::{GameResources, GameState},
    render::{
        mesh::Mesh,
        renderer::{Renderer, Viewport},
        setup_2d_rendering,
    },
};

pub struct UIRenderer {
    mesh: Mesh,
}

impl UIRenderer {
    pub fn new(game: &GameState, resources: &GameResources, viewport: &Viewport) -> Self {
        Self {
            mesh: Self::build_mesh(game, resources, viewport),
        }
    }

    fn crosshair_center(viewport: &Viewport) -> Vec2 {
        Vec2::new(viewport.width as f32 / 2.0, viewport.height as f32 / 2.0)
    }

    fn fps_center(viewport: &Viewport) -> Vec2 {
        Vec2::new(viewport.width as f32 - 50.0, 50.0)
    }

    fn build_mesh(game: &GameState, resources: &GameResources, viewport: &Viewport) -> Mesh {
        let crosshair_vertices = resources.get_crosshair_vertices(Self::crosshair_center(viewport));
        let fps_vertices = resources.get_fps_vertices(Self::fps_center(viewport), *game.state.fps);
        let selected_block_vertices = resources.get_vertices_for_block_face(
            *game.state.selected_block_type,
            Vec2::new(50., viewport.height as f32 - 50.),
        );

        let vertices: Vec<_> = crosshair_vertices
            .into_iter()
            .chain(fps_vertices)
            .chain(selected_block_vertices)
            .collect();

        Mesh::new(&vertices, Mat4::IDENTITY, resources.atlas.texture)
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn rebuild_mesh_if_dirty(
        &mut self,
        game: &GameState,
        resources: &GameResources,
        viewport: &Viewport,
    ) -> bool {
        if game.state.selected_block_type.is_dirty() || game.state.fps.is_dirty() {
            self.mesh = Self::build_mesh(game, resources, viewport);
            true
        } else {
            false
        }
    }

    pub fn render(
        &mut self,
        game: &GameState,
        resources: &GameResources,
        renderer: &Renderer,
        viewport: &Viewport,
    ) {
        setup_2d_rendering();
        self.rebuild_mesh_if_dirty(game, resources, viewport);
        renderer.render_mesh_2d(&self.mesh, viewport);
    }
}
