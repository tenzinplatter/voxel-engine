use glam::{IVec3, Vec3};

use crate::{
    engine::{block::BlockType, game::GameResources},
    physics::PhysicsBody,
    render::vertex::VertexTex,
};

pub(crate) const FACE_NORMALS: [IVec3; 6] = [
    IVec3::NEG_Z,
    IVec3::Z,
    IVec3::NEG_X,
    IVec3::X,
    IVec3::NEG_Y,
    IVec3::Y,
];

#[derive(Debug)]
pub(crate) struct Voxel {
    pub(crate) body: PhysicsBody,
    block_type: BlockType,
}

impl Voxel {
    /// Creates a new voxel at the given integer position.
    pub(crate) fn new(position: IVec3, block_type: BlockType) -> Self {
        Voxel {
            body: PhysicsBody::new(position.as_vec3(), Vec3::ONE),
            block_type,
        }
    }

    /// Returns the vertices of every face not pointing along one of `hidden_faces`,
    /// two triangles (6 vertices) per face.
    pub(crate) fn get_vertices(
        &self,
        resources: &GameResources,
        hidden_faces: &[IVec3],
    ) -> Vec<VertexTex> {
        let pos = self.body.position;

        // UV coordinates for each corner of a face
        let uvs = resources
            .atlas
            .textures
            .get(self.block_type.as_str())
            .unwrap_or_else(|| {
                panic!(
                    "Block type: {} doesn't exist in atlas",
                    self.block_type.as_str()
                )
            })
            .to_uvs();

        FACE_NORMALS
            .into_iter()
            .filter(|normal| !hidden_faces.contains(normal))
            .flat_map(|normal| {
                let corners = face_corners(normal);
                // Counter-clockwise when viewed from outside
                [0, 3, 2, 2, 1, 0].map(|i| VertexTex::new(corners[i] + pos, uvs[i]))
            })
            .collect()
    }
}

/// Corners of the unit-cube face pointing along `normal`, in the same order as the atlas UVs:
/// [top_left, top_right, bottom_right, bottom_left] when viewed from outside.
fn face_corners(normal: IVec3) -> [Vec3; 4] {
    let normal = normal.as_vec3();
    let up = if normal.y == 0.0 { Vec3::Y } else { Vec3::Z };
    let right = up.cross(normal);
    let centre = Vec3::splat(0.5) + normal * 0.5;
    let (r, u) = (right * 0.5, up * 0.5);
    [
        centre - r + u,
        centre + r + u,
        centre + r - u,
        centre - r - u,
    ]
}
