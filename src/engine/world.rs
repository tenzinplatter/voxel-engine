use core::panic;
use std::{collections::HashMap, ops::Div};

use glam::{IVec2, IVec3, Mat4, Vec3Swizzles};
use itertools::iproduct;
use noise::{NoiseFn, Perlin};

use crate::{
    engine::{block::BlockType, game::GameResources, voxel::Voxel},
    physics::{PhysicsBody, colliding_with},
    render::mesh::Mesh,
    utils::tracked_map::TrackedHashMap,
};

const CHUNK_SIZE: i32 = 16;

pub struct World {
    pub voxels: TrackedHashMap<IVec3, Voxel>,
    pub chunk_meshes: Vec<Mesh>,
}

pub struct ChunkData<'a> {
    voxels: Vec<&'a Voxel>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Rect {
    bottom_left: IVec2,
    top_right: IVec2,
}

impl World {
    /// Rebuilds the world's mesh from all voxels, optionally using a new texture.
    pub fn rebuild_mesh(&mut self, resources: &GameResources) {
        let chunks = self.get_chunks();

        self.chunk_meshes = chunks
            .into_iter()
            .map(|chunk| {
                let vertices: Vec<_> = chunk
                    .voxels
                    .iter()
                    .flat_map(|vox| vox.get_vertices(resources))
                    .collect();

                Mesh::new(&vertices, Mat4::IDENTITY, resources.atlas.texture)
            })
            .collect();
    }

    pub fn get_chunks(&self) -> Vec<ChunkData<'_>> {
        fn bucket_from_pos<'a, 'b>(
            chunks: &'a mut HashMap<Rect, Vec<&'b Voxel>>,
            pos: IVec2,
        ) -> &'a mut Vec<&'b Voxel> {
            let chunk_bottom_left = pos.map(|v| v / CHUNK_SIZE * CHUNK_SIZE);
            let chunk = Rect {
                bottom_left: chunk_bottom_left,
                top_right: chunk_bottom_left + CHUNK_SIZE,
            };

            chunks
                .get_mut(&chunk)
                .expect("Chunks should contain entries for every chunk that contains a voxel")
        }

        let chunk_rects = self.get_chunk_positions();
        let mut chunks: HashMap<Rect, Vec<&Voxel>> =
            chunk_rects.into_iter().map(|r| (r, Vec::new())).collect();

        for vox in self.voxels.values() {
            let pos = vox.body.position.xz().as_ivec2();
            let bucket = bucket_from_pos(&mut chunks, pos);
            bucket.push(vox);
        }

        chunks.into_values().map(|voxels| ChunkData { voxels })
            .collect()
    }

    fn get_chunk_positions(&self) -> Vec<Rect> {
        let world_border = self.get_world_border();

        let top_right = world_to_chunk_pos(world_border.top_right);
        let bottom_left = world_to_chunk_pos(world_border.bottom_left);

        let x_range = bottom_left.x..top_right.x;
        let y_range = bottom_left.y..top_right.y;

        let bottom_lefts: Vec<_> = iproduct!(x_range, y_range).collect();

        bottom_lefts
            .into_iter()
            .map(|(x, y)| {
                let bottom_left = IVec2::new(x, y) * 16;
                Rect {
                    bottom_left,
                    top_right: bottom_left + 16,
                }
            })
            .collect()
    }

    fn get_world_border(&self) -> Rect {
        if self.voxels.len() < 2 {
            panic!("World to small to generate border.");
        }

        let mut top_right = IVec2::MIN;
        let mut bottom_left = IVec2::MAX;

        for voxel in self.voxels.values() {
            let IVec2 { x, y } = voxel.body.position.xz().as_ivec2();

            top_right.x = i32::max(top_right.x, x);
            top_right.y = i32::max(top_right.y, y);

            bottom_left.x = i32::min(bottom_left.x, x);
            bottom_left.y = i32::min(bottom_left.y, y);
        }

        Rect {
            top_right,
            bottom_left,
        }
    }

    /// Adds a voxel at the given position, returning the old value if one existed.
    pub fn set_voxel(&mut self, pos: IVec3, block_type: BlockType) -> Option<Voxel> {
        self.voxels.insert(pos, Voxel::new(pos, block_type))
    }

    /// Removes the voxel at the given position, returning it if it existed.
    pub fn remove_voxel(&mut self, pos: &IVec3) -> Option<Voxel> {
        self.voxels.remove(pos)
    }

    /// Checks if the given physics body is colliding with any voxel in the world.
    pub fn is_colliding(&self, other: &PhysicsBody) -> bool {
        // TODO: extrusion or something so we cant phase through voxels when moving quickly
        // TODO: optimize to not check every square
        self.voxels.values().any(|v| colliding_with(&v.body, other))
    }

    pub fn from_noise() -> World {
        let perlin = Perlin::new(1);
        let scale = 0.05;

        let voxels = (-32..32)
            .flat_map(|z| {
                (-32..32).flat_map(move |x| {
                    let noise = perlin.get([x as f64 * scale, z as f64 * scale]);
                    let y = (noise * 10.0) as i32;
                    let stone_start = y - 3;
                    (-10..y).map(move |y| {
                        let pos = IVec3::new(x, y, z);
                        (
                            pos,
                            Voxel::new(
                                pos,
                                if y < stone_start {
                                    BlockType::Stone
                                } else {
                                    BlockType::Dirt
                                },
                            ),
                        )
                    })
                })
            })
            .collect();

        World {
            voxels,
            chunk_meshes: Vec::new(),
        }
    }
}

impl Default for World {
    fn default() -> Self {
        let voxels = (-32..32)
            .flat_map(|z| {
                (-32..32).map(move |x| {
                    let pos = IVec3::new(x, 0, z);
                    (pos, Voxel::new(pos, BlockType::Dirt))
                })
            })
            .collect();

        World {
            voxels,
            chunk_meshes: Vec::new(),
        }
    }
}

fn world_to_chunk_pos(vec: IVec2) -> IVec2 {
    vec.as_vec2().div(16.0).ceil().as_ivec2()
}
