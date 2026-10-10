use std::collections::HashMap;

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
const WORLD_SIZE: i32 = 32;

enum DirtyVoxel<'a> {
    Exists(&'a Voxel),
    Removed(IVec3),
}

impl<'a> DirtyVoxel<'a> {
    fn xz(&self) -> IVec2 {
        match self {
            DirtyVoxel::Exists(voxel) => voxel.body.position.xz().as_ivec2(),
            DirtyVoxel::Removed(ivec3) => ivec3.xz(),
        }
    }
}

pub struct ChunkData<'a> {
    border: ChunkBorder,
    voxels: Vec<&'a Voxel>,
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ChunkBorder {
    top_right: IVec2,
    bottom_left: IVec2,
}

impl ChunkBorder {
    fn expand(&mut self, other: Self) {
        self.top_right.x = i32::max(self.top_right.x, other.top_right.x);
        self.top_right.y = i32::max(self.top_right.y, other.top_right.y);

        self.bottom_left.x = i32::min(self.bottom_left.x, other.bottom_left.x);
        self.bottom_left.y = i32::min(self.bottom_left.y, other.bottom_left.y);
    }
}

pub struct World {
    pub voxels: TrackedHashMap<IVec3, Voxel>,
    pub chunk_meshes: HashMap<ChunkBorder, Mesh>,
}

impl World {
    /// Rebuilds the world's mesh from all voxels, optionally using a new texture.
    pub fn rebuild_dirty_chunks(&mut self, resources: &GameResources) {
        let chunk_mesh = |chunk: &ChunkData| -> Mesh {
            let vertices: Vec<_> = chunk
                .voxels
                .iter()
                .flat_map(|vox| vox.get_vertices(resources))
                .collect();

            Mesh::new(&vertices, Mat4::IDENTITY, resources.atlas.texture)
        };

        let chunks = self.get_dirty_chunks();
        let meshes_by_border: Vec<_> = chunks
            .iter()
            .map(|chunk| (chunk.border, chunk_mesh(chunk)))
            .collect();

        for (border, mesh) in meshes_by_border {
            if let Some(old) = self.chunk_meshes.get_mut(&border) {
                *old = mesh;
            } else {
                self.chunk_meshes.insert(border, mesh);
            }
        }
    }

    pub fn get_dirty_chunks(&self) -> Vec<ChunkData<'_>> {
        let chunk_rects = self.get_dirty_chunk_positions();
        let mut chunks: HashMap<ChunkBorder, Vec<DirtyVoxel>> =
            chunk_rects.into_iter().map(|r| (r, Vec::new())).collect();

        for vox in self.dirty_voxels() {
            let pos = vox.xz();
            let chunk = chunk_pos_to_world_coords(world_to_chunk_pos(pos));
            let bucket = chunks
                .get_mut(&chunk)
                .expect("all chunks should have map entries here");
            bucket.push(vox);
        }

        let dirty_chunks: Vec<_> = chunks
            .iter()
            .filter(|(_, voxels)| !voxels.is_empty())
            .map(|(pos, _)| pos)
            .collect();

        let mut chunks: HashMap<ChunkBorder, Vec<&Voxel>> =
            dirty_chunks.into_iter().map(|r| (*r, Vec::new())).collect();

        for vox in self.voxels.values() {
            let pos = vox.body.position.xz().as_ivec2();
            let chunk = chunk_pos_to_world_coords(world_to_chunk_pos(pos));
            if let Some(bucket) = chunks.get_mut(&chunk) {
                bucket.push(vox);
            }
        }

        chunks
            .into_iter()
            .map(|(border, voxels)| ChunkData { border, voxels })
            .collect()
    }

    fn get_dirty_chunk_positions(&self) -> Vec<ChunkBorder> {
        let Some(world_border) = self.get_dirty_world_border() else {
            return Vec::new();
        };

        let top_right = world_to_chunk_pos(world_border.top_right) + 1;
        let bottom_left = world_to_chunk_pos(world_border.bottom_left);

        let x_range = bottom_left.x..top_right.x;
        let y_range = bottom_left.y..top_right.y;

        let bottom_lefts: Vec<_> = iproduct!(x_range, y_range).collect();

        bottom_lefts
            .into_iter()
            .map(IVec2::from)
            .map(chunk_pos_to_world_coords)
            .collect()
    }

    fn get_dirty_world_border(&self) -> Option<ChunkBorder> {
        let dirty = self.dirty_voxels();
        if dirty.is_empty() {
            return None;
        }

        let mut dirty_border = ChunkBorder::default();
        for voxel in dirty {
            let chunk_idx = world_to_chunk_pos(voxel.xz());
            dirty_border.expand(chunk_pos_to_world_coords(chunk_idx));
        }

        Some(dirty_border)
    }

    fn dirty_voxels(&self) -> Vec<DirtyVoxel<'_>> {
        self.voxels
            .dirty()
            .map(|k| match self.voxels.get(k) {
                Some(vox) => DirtyVoxel::Exists(vox),
                None => DirtyVoxel::Removed(*k),
            })
            .collect()
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

        let voxels = (-WORLD_SIZE..WORLD_SIZE)
            .flat_map(|z| {
                (-WORLD_SIZE..WORLD_SIZE).flat_map(move |x| {
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
            chunk_meshes: HashMap::new(),
        }
    }
}

impl Default for World {
    fn default() -> Self {
        let voxels = (-WORLD_SIZE..WORLD_SIZE)
            .flat_map(|z| {
                (-WORLD_SIZE..WORLD_SIZE).map(move |x| {
                    let pos = IVec3::new(x, 0, z);
                    (pos, Voxel::new(pos, BlockType::Dirt))
                })
            })
            .collect();

        World {
            voxels,
            chunk_meshes: HashMap::new(),
        }
    }
}

fn world_to_chunk_pos(vec: IVec2) -> IVec2 {
    vec.div_euclid(IVec2::splat(CHUNK_SIZE))
}

fn chunk_pos_to_world_coords(vec: IVec2) -> ChunkBorder {
    let bottom_left = vec * 16;
    ChunkBorder {
        bottom_left,
        top_right: bottom_left + 16,
    }
}
