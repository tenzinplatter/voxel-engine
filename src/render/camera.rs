use glam::{Mat4, Vec3};

const WORLD_UP: Vec3 = Vec3::Y;

pub(crate) struct Camera {
    // Position
    pub(crate) position: Vec3,

    // Orientation (Euler angles in radians)
    pub(crate) yaw: f32,   // Rotation around Y axis (left/right)
    pub(crate) pitch: f32, // Rotation around X axis (up/down)

    // Camera vectors (computed from yaw/pitch)
    pub(crate) front: Vec3, // Direction camera is looking
    pub(crate) right: Vec3, // Right vector perpendicular to front
    pub(crate) up: Vec3,    // Up vector perpendicular to front and right
}

impl Camera {
    /// Creates a camera looking at a specific point.
    pub(crate) fn looking_at(position: Vec3, target: Vec3) -> Self {
        let direction = (target - position).normalize();

        // Convert direction to yaw/pitch
        let yaw = direction.z.atan2(direction.x);
        let pitch = direction.y.asin();

        let mut camera = Self {
            position,
            yaw,
            pitch,
            front: direction,
            right: Vec3::X,
            up: Vec3::Y,
        };
        camera.update_vectors();
        camera
    }

    /// Returns the view matrix for this camera.
    pub(crate) fn view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.front, self.up)
    }

    /// Recomputes front, right, and up vectors from yaw and pitch.
    pub(crate) fn update_vectors(&mut self) {
        // Calculate new front vector from yaw and pitch
        self.front = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        // Recalculate right and up vectors
        self.right = self.front.cross(WORLD_UP).normalize();
        self.up = self.right.cross(self.front).normalize();
    }
}
