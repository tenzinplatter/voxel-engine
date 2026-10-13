use glam::{BVec3A, IVec3, Vec3A, Vec3Swizzles};

/// The Digital-Differential-Analyzer (DDA) Algorithm for a 3D Grid / Voxels.
///
/// ```ignore
/// use glam::Vec3A;
/// const MAX_STEPS: usize = 128;
/// let ro = Vec3A::new(0.0, 1.8, 0.0); // origin
/// let rd = Vec3A::new(0.0, 0.0, 1.0); // direction
/// let mut dda = DDAState::from_pos_and_dir(ro, rd);
///
/// for _ in 0..MAX_STEPS {
///     // check if dda.next_voxelpos is 'inside' a voxel
///     // use the dda.hit_* functions here
///     if false { break; }
///     // no hit? step the ray
///     dda.step_mut();
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub(crate) struct DDAState {
    /// The current largest component of/to the next boundary.
    max_boundary_mask: BVec3A,

    /// Per-component signum to the next voxel position.
    diff_voxelpos: IVec3, // = signum(dir)

    /// The next voxel position to be visited.
    pub(crate) next_voxel_pos: IVec3,

    /// Per-component distance to next boundary plane.
    diff_boundary: Vec3A,

    /// The distances to the next voxel boundary on the X/Y/Z axes.
    next_boundary: Vec3A,
}

impl DDAState {
    #[inline(always)]
    pub(crate) fn from_pos_and_dir(ray_origin: glam::Vec3A, ray_direction: glam::Vec3A) -> Self {
        let ray_origin_grid = ray_origin.floor();
        let ray_origin_grid_i = ray_origin_grid.as_ivec3();

        let mut ray_direction = ray_direction;
        if ray_direction.x == 0.0 {
            ray_direction.x = 0.00001;
        }
        if ray_direction.y == 0.0 {
            ray_direction.y = 0.00001;
        }
        if ray_direction.z == 0.0 {
            ray_direction.z = 0.00001;
        }
        let ray_direction = ray_direction.normalize();

        let ray_sign = ray_direction.signum(); // Step
        let ray_dir_inv = Vec3A::ONE / ray_direction; // invDir

        // vec3 sideDist = (sign(rayDir) * (vec3(mapPos) - rayPos) + (sign(rayDir) * 0.5) + 0.5) * deltaDist;
        let ray_dist = ray_dir_inv.abs();
        let next_dist =
            (ray_sign * (ray_origin_grid - ray_origin) + (ray_sign * 0.5) + 0.5) * ray_dist;

        Self {
            // Values that stay fixed.
            diff_boundary: ray_dist,
            diff_voxelpos: ray_sign.as_ivec3(),

            max_boundary_mask: BVec3A::FALSE,
            next_boundary: next_dist,
            next_voxel_pos: ray_origin_grid_i,
        }
    }

    #[inline(always)]
    pub(crate) fn step_mut(&mut self) {
        // Find the longest component of `next_boundary` and store it.
        self.max_boundary_mask = self
            .next_boundary
            .xyz()
            .cmple(self.next_boundary.yzx().min(self.next_boundary.zxy()));
        self.next_voxel_pos += self.diff_voxelpos * IVec3::from(self.max_boundary_mask);
        self.next_boundary += self.diff_boundary * Vec3A::from(self.max_boundary_mask);
    }

    #[inline(always)]
    pub(crate) fn hit_distance(&self) -> f32 {
        ((self.next_boundary - self.diff_boundary) * Vec3A::from(self.max_boundary_mask))
            .element_sum()
    }

    /// Normal of the hit on the next voxels boundary.
    #[inline(always)]
    pub(crate) fn hit_normal(&self) -> Vec3A {
        -self.diff_voxelpos.as_vec3a() * Vec3A::from(self.max_boundary_mask)
    }
}
