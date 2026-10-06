use anyhow::Result;
use beryllium::{
    Sdl,
    events::{self},
};
use glam::{IVec3, Vec2, Vec3};

use crate::{
    engine::{block::BlockType, world::World},
    input::InputState,
    physics::{colliding_with_voxel_from_pos, dda::get_looking_at_vox_pos, hit_info::HitInfo},
    player::Player,
    render::{
        atlas::{TEXTURE_SIZE_PX, TextureAtlas},
        vertex::Vertex2D,
    },
    utils::{tracked::Tracked, types::Seconds},
};

pub struct GameState {
    pub state: State,
    pub world: World,
    pub player: Player,
    pub input_state: InputState,
}

pub struct GameResources {
    pub atlas: TextureAtlas,
}

#[derive(Default)]
pub struct State {
    pub looking_at_vox_pos: Option<IVec3>,
    pub selected_block_type: Tracked<BlockType>,
    pub fps: Tracked<u32>,
}

impl GameResources {
    pub fn build() -> Result<Self> {
        Ok(Self {
            atlas: TextureAtlas::try_parse_atlas()?,
        })
    }

    pub fn get_vertices_for_block_face(
        &self,
        block_type: BlockType,
        center: Vec2,
    ) -> [Vertex2D; 6] {
        let size = TEXTURE_SIZE_PX as f32;

        let uvs = self
            .atlas
            .textures
            .get(block_type.as_str())
            .unwrap_or_else(|| panic!("No texture for block type: {}", block_type.as_str()))
            .to_uvs();

        vertices_from_center_and_size(center, size, uvs)
    }

    pub fn get_crosshair_vertices(&self, center: Vec2) -> [Vertex2D; 6] {
        const CROSSHAIR_SIZE: f32 = 16.0;
        let uvs = self
            .atlas
            .textures
            .get("crosshair")
            .expect("Crosshair texture missing from atlas")
            .to_uvs();

        vertices_from_center_and_size(center, CROSSHAIR_SIZE, uvs)
    }

    pub fn get_fps_vertices(&self, center: Vec2, fps: u32) -> Vec<Vertex2D> {
        let s = format!("{fps}");
        let digits = s.chars().map(|c| {
            c.to_digit(10)
                .expect("There shouldn't be any non digit chars here")
        });
        digits
            .enumerate()
            .flat_map(|(i, d)| {
                let center = center.with_x(center.x + (i * 12) as f32);
                self.get_digit_vertices(d, center)
            })
            .collect::<Vec<_>>()
    }

    fn get_digit_vertices(&self, digit: u32, center: Vec2) -> [Vertex2D; 6] {
        assert!(
            matches!(digit, 0..=9),
            "Should not pass a value not in 0..=9 to get_digit_vertices: {digit}"
        );

        const DIGIT_SIZE: f32 = 16.0;
        let uvs = self
            .atlas
            .textures
            .get(&format!("digit_{digit}"))
            .expect("Crosshair texture missing from atlas")
            .to_uvs();

        vertices_from_center_and_size(center, DIGIT_SIZE, uvs)
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            state: State::default(),
            world: World::from_noise(),
            player: Player::new(Vec3::new(-3.0, 2.0, -3.0)),
            input_state: InputState::default(),
        }
    }
}

impl GameState {
    pub fn update_fps(&mut self, fps: u32) {
        if fps.abs_diff(*self.state.fps) > 3 {
            *self.state.fps = fps;
        }
    }

    pub fn reset(&mut self) {
        self.input_state.reset();
    }

    /// Processes input events, updating the player and input state accordingly.
    /// Returns whether a quit event was received.
    pub fn process_input_events(&mut self, sdl: &Sdl) -> bool {
        while let Some(event) = sdl.poll_events() {
            match event {
                (events::Event::Quit, _) => return true,
                (
                    events::Event::Key {
                        keycode, pressed, ..
                    },
                    _,
                ) => {
                    self.input_state.set_key(keycode, pressed);
                }
                (
                    events::Event::MouseMotion {
                        x_delta, y_delta, ..
                    },
                    _,
                ) => {
                    self.player.process_mouse(x_delta as f32, -y_delta as f32);
                }
                (
                    events::Event::MouseButton {
                        button, pressed, ..
                    },
                    _,
                ) => {
                    self.input_state.set_mouse_button(button, pressed);
                }
                _ => {}
            }
        }

        if self.input_state.number_key(1).just_pressed {
            self.state.selected_block_type.set(BlockType::Dirt);
        } else if self.input_state.number_key(2).just_pressed {
            self.state.selected_block_type.set(BlockType::Stone);
        }

        let hit_info = get_looking_at_vox_pos(&self.world, &self.player);
        self.state.looking_at_vox_pos = hit_info.map(|hit| hit.pos);
        if let Some(hit_info) = hit_info {
            self.handle_mouse_presses(&hit_info);
        }

        self.input_state.reset_mouse_buttons();

        false
    }

    pub fn update_player_and_world(&mut self, delta_time: Seconds) {
        self.player
            .step(&self.world, delta_time, &mut self.input_state);
    }

    pub fn handle_mouse_presses(&mut self, hit_info: &HitInfo) {
        if self.input_state.mb3.just_pressed {
            self.try_place_block(hit_info);
        }

        if self.input_state.mb1.just_pressed {
            self.try_remove_block(hit_info);
        }

        self.input_state.reset_mouse_buttons();
    }

    fn try_remove_block(&mut self, hit_info: &HitInfo) {
        let to_remove = hit_info.pos;
        self.world.voxels.remove(&to_remove);
    }

    fn try_place_block(&mut self, hit_info: &HitInfo) -> bool {
        let to_place = hit_info.pos + hit_info.normal;
        if self.world.voxels.contains_key(&to_place)
            || colliding_with_voxel_from_pos(&self.player.body, to_place.as_vec3())
        {
            return false;
        }

        self.world
            .set_voxel(to_place, *self.state.selected_block_type);
        true
    }
}

pub fn vertices_from_center_and_size(center: Vec2, size: f32, uvs: [Vec2; 4]) -> [Vertex2D; 6] {
    let half = Vec2::splat(size / 2.0);
    let corners = [
        Vec2::new(-1., -1.),
        Vec2::new(1., -1.),
        Vec2::ONE,
        Vec2::new(-1., 1.),
    ]
    .map(|sign| center + sign * half);

    [0, 1, 2, 2, 3, 0].map(|i| Vertex2D {
        position: corners[i],
        tex: uvs[i],
    })
}
