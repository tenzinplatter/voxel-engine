use glam::Vec3;

pub(crate) mod dda;
mod dda_other;
pub(crate) mod hit_info;
pub(crate) mod movement;

pub(crate) const PHYSICS_DT: f32 = 1. / 120.;
const GRAVITY: f32 = -9.81;

#[derive(Clone, Debug)]
pub(crate) struct PhysicsBody {
    pub(crate) position: Vec3,
    pub(crate) velocity: Vec3,
    pub(crate) size: Vec3,
    pub(crate) accumulator: f32,
}

impl PhysicsBody {
    /// Creates a new physics body at the given position and size.
    pub(crate) fn new(position: Vec3, size: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            size,
            accumulator: 0.,
        }
    }

    /// Steps the body based on its currently set velocity and the given `dt`
    fn step(&mut self, dt: f32) {
        self.position += self.velocity * dt;
    }
}

pub(crate) fn colliding_with_voxel_from_pos(p: &PhysicsBody, vox: Vec3) -> bool {
    let vox_body = PhysicsBody::new(vox, Vec3::ONE);
    colliding_with(p, &vox_body)
}

/// Checks whether two physics bodies are colliding using AABB collision detection.
/// TODO: when using this do a binary search from start time to end time to see how far the object
/// can be moved before colliding, ~5 steps is probably good
pub(crate) fn colliding_with(a: &PhysicsBody, b: &PhysicsBody) -> bool {
    a.position.x < b.position.x + b.size.x
        && a.position.x + a.size.x > b.position.x
        && a.position.y < b.position.y + b.size.y
        && a.position.y + a.size.y > b.position.y
        && a.position.z < b.position.z + b.size.z
        && a.position.z + a.size.z > b.position.z
}
