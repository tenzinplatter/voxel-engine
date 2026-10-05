use std::rc::Rc;

use glam::{Mat4, Vec2};

use crate::{
    engine::{
        block::BlockType,
        game::{GameResources, GameState},
    },
    get_crosshair_vertices, get_fps_vertices,
    render::{
        mesh::Mesh,
        renderer::{Renderer, Viewport},
        setup_2d_rendering,
        vertex::Vertex2D,
    },
    utils::tracked::Tracked,
};

pub struct UIRenderer {
    mesh: Mesh,
    selected_block_type: Rc<Tracked<BlockType>>,
}

impl UIRenderer {
    pub fn new(game: &GameState, resources: &GameResources, viewport: &Viewport) -> Self {
        Self {
            mesh: Self::build_mesh(game, resources, viewport),
            selected_block_type: game.state.selected_block_type.clone(),
        }
    }

    fn build_mesh(game: &GameState, resources: &GameResources, viewport: &Viewport) -> Mesh {
        let mut vertices: Vec<Vertex2D> = get_crosshair_vertices(resources, viewport).to_vec();
        vertices.extend(get_fps_vertices(resources, viewport, *game.state.fps.get()));
        vertices.extend(resources.get_vertices_for_block_face(
            *game.state.selected_block_type.get(),
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
        if self.selected_block_type.take_dirty().is_some() || game.state.fps.take_dirty().is_some()
        {
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
