use std::collections::HashMap;

use glam::{IVec2, IVec3, Mat4, Vec3, Vec3Swizzles};
use noise::{NoiseFn, Perlin};

use crate::{
    engine::{
        block::BlockType,
        game::GameResources,
        voxel::{FACE_NORMALS, Voxel},
    },
    physics::{PhysicsBody, colliding_with},
    render::mesh::Mesh,
    utils::tracked_map::TrackedHashMap,
};

const CHUNK_SIZE: i32 = 16;
const WORLD_SIZE: i32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ChunkPos(IVec2);

impl ChunkPos {
    fn containing(voxel: IVec3) -> Self {
        Self(voxel.xz().div_euclid(IVec2::splat(CHUNK_SIZE)))
    }
}

pub(crate) struct World {
    pub(crate) voxels: TrackedHashMap<IVec3, Voxel>,
    pub(crate) chunk_meshes: HashMap<ChunkPos, Mesh>,
}

impl World {
    /// Rebuilds the mesh of every chunk containing a voxel changed since the last frame.
    pub(crate) fn rebuild_dirty_chunks(&mut self, resources: &GameResources) {
        let rebuilt: Vec<_> = self
            .dirty_chunks()
            .into_iter()
            .map(|(chunk, voxels)| (chunk, self.chunk_mesh(resources, &voxels)))
            .collect();

        for (chunk, mesh) in rebuilt {
            match mesh {
                Some(mesh) => self.chunk_meshes.insert(chunk, mesh),
                None => self.chunk_meshes.remove(&chunk),
            };
        }
    }

    /// Directions of the faces of the voxel at `pos` that are covered by an adjacent voxel.
    fn hidden_faces(&self, pos: IVec3) -> Vec<IVec3> {
        FACE_NORMALS
            .into_iter()
            .filter(|&normal| self.voxels.contains_key(&(pos + normal)))
            .collect()
    }

    /// Groups voxels by chunk for every chunk containing, or bordering, a changed voxel.
    fn dirty_chunks(&self) -> HashMap<ChunkPos, Vec<(IVec3, &Voxel)>> {
        // A change can expose or hide faces of voxels in a neighbouring chunk
        let mut chunks: HashMap<ChunkPos, Vec<(IVec3, &Voxel)>> = self
            .voxels
            .dirty()
            .flat_map(|&pos| {
                std::iter::once(pos)
                    .chain(FACE_NORMALS.map(|normal| pos + normal))
                    .map(|pos| (ChunkPos::containing(pos), Vec::new()))
            })
            .collect();

        for (&pos, voxel) in self.voxels.iter() {
            if let Some(bucket) = chunks.get_mut(&ChunkPos::containing(pos)) {
                bucket.push((pos, voxel));
            }
        }

        chunks
    }

    fn chunk_mesh(
        &self,
        resources: &GameResources,
        voxels: &[(IVec3, &Voxel)],
    ) -> Option<Mesh> {
        if voxels.is_empty() {
            return None;
        }

        let vertices: Vec<_> = voxels
            .iter()
            .flat_map(|&(pos, vox)| {
                vox.get_vertices(resources, &self.hidden_faces(pos))
            })
            .collect();

        Some(Mesh::new(
            &vertices,
            Mat4::IDENTITY,
            resources.atlas.texture,
        ))
    }

    /// Adds a voxel at the given position, returning the old value if one existed.
    pub(crate) fn set_voxel(&mut self, pos: IVec3, block_type: BlockType) -> Option<Voxel> {
        self.voxels.insert(pos, Voxel::new(pos, block_type))
    }

    /// Checks if the given physics body is colliding with any voxel in the world.
    pub(crate) fn is_colliding(&self, other: &PhysicsBody) -> bool {
        // TODO: extrusion or something so we cant phase through voxels when moving quickly
        // TODO: optimize to not check every square
        self.voxels.values().any(|v| colliding_with(&v.body, other))
    }

    pub(crate) fn from_noise() -> World {
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
