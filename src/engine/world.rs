use glam::{IVec3, Mat4};
use noise::{NoiseFn, Perlin};

use crate::{
    engine::{block::BlockType, game::GameResources, voxel::Voxel},
    physics::{PhysicsBody, colliding_with},
    render::mesh::Mesh,
    utils::tracked_map::TrackedHashMap,
};

pub struct World {
    pub voxels: TrackedHashMap<IVec3, Voxel>,
    pub mesh: Option<Mesh>,
}

impl World {
    /// Rebuilds the world's mesh from all voxels, optionally using a new texture.
    pub fn rebuild_mesh(&mut self, resources: &GameResources) {
        // TODO: presize this to correct size
        let mut vertcies = vec![];

        // voxels positions are top x, z corner
        for vox in self.voxels.values() {
            vertcies.extend(vox.get_vertices(resources));
        }

        self.mesh = Some(Mesh::new(
            &vertcies,
            Mat4::IDENTITY,
            resources.atlas.texture,
        ));
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
            mesh: Default::default(),
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
            mesh: Default::default(),
        }
    }
}
