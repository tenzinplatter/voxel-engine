use glam::{Mat4, Vec2};

use crate::{
    engine::game::{GameResources, GameState},
    get_crosshair_vertices, get_fps_vertices,
    render::{
        mesh::Mesh,
        renderer::{Renderer, Viewport},
        setup_2d_rendering,
        vertex::Vertex2D,
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

    fn build_mesh(game: &GameState, resources: &GameResources, viewport: &Viewport) -> Mesh {
        let mut vertices: Vec<Vertex2D> = get_crosshair_vertices(resources, viewport).to_vec();
        vertices.extend(get_fps_vertices(resources, viewport, *game.state.fps));
        vertices.extend(resources.get_vertices_for_block_face(
            *game.state.selected_block_type,
            Vec2::new(50., viewport.height as f32 - 50.),
        ));

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
