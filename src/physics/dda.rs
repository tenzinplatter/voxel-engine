use glam::{IVec3, Vec3};

use crate::{
    engine::world::World,
    physics::{dda_other::DDAState, hit_info::{HitFace, HitInfo}},
    player::{Player, DEFAULT_PLAYER_REACH},
};

fn get_dda_tdelta(dir: Vec3) -> Vec3 {
    /// calculates the tdelta for a single dimension
    fn helper(d: f32) -> f32 {
        if d != 0.0 { (1.0 / d).abs() } else { f32::MAX }
    }

    Vec3::new(helper(dir.x), helper(dir.y), helper(dir.z))
}

fn get_dda_tmax(dir: Vec3, origin: Vec3, step: IVec3) -> Vec3 {
    /// calculates the tmax for a single dimension
    fn helper(dir: f32, origin: f32, step: i32) -> f32 {
        if dir != 0.0 {
            let voxel_border = if step > 0 {
                origin.floor() + 1.0
            } else {
                origin.floor()
            };
            (voxel_border - origin) / dir
        } else {
            f32::MAX
        }
    }

    Vec3::new(
        helper(dir.x, origin.x, step.x),
        helper(dir.y, origin.y, step.y),
        helper(dir.z, origin.z, step.z),
    )
}

pub fn get_looking_at_vox_pos(world: &World, player: &Player) -> Option<HitInfo> {
    let mut state = DDAState::from_pos_and_dir(
        player.camera.position.into(),
        player.camera.front.normalize().into(),
    );

    while state.hit_distance() < DEFAULT_PLAYER_REACH {
        // step first to skip starting voxel
        state.step_mut();

        let curr_pos = state.next_voxelpos;
        if world.voxels.contains_key(&curr_pos) {
            let normal = state.hit_normal();
            let face = if normal.x < 0.0 {
                HitFace::NegX
            } else if normal.x > 0.0 {
                HitFace::PosX
            } else if normal.y < 0.0 {
                HitFace::NegY
            } else if normal.y > 0.0 {
                HitFace::PosY
            } else if normal.z < 0.0 {
                HitFace::NegZ
            } else {
                HitFace::PosZ
            };

            return Some(HitInfo::new(curr_pos, face));
        }
    }

    None
}

/// Return the voxel that the player is looking at within the players reach, if there is one
pub fn get_looking_at_vox_pos_old(world: &World, player: &Player) -> Option<HitInfo> {
    let origin = {
        let pos = player.camera.position;
        IVec3::new(pos.x.floor() as i32, pos.y.floor() as i32, pos.z.floor() as i32)
    };

    let direction = player.camera.front.normalize();
    let step = IVec3::new(
        if direction.x > 0.0 { 1 } else { -1 },
        if direction.y > 0.0 { 1 } else { -1 },
        if direction.z > 0.0 { 1 } else { -1 },
    );

    let tdelta = get_dda_tdelta(direction);
    let mut tmax = get_dda_tmax(direction, player.camera.position, step);

    // scalar representation of how many |direction| lengths we've traveled across the ray
    // NOT in world units
    // i.e. our current pos = origin + direction * t
    let mut t = 0.;
    let mut last_step: Option<HitFace> = None;
    let mut curr_pos = origin;

    // we can just compare t since direction is normalized
    while t < DEFAULT_PLAYER_REACH {
        // skip this check on the first iteration as last step won't be set yet
        // should be safe as for this to be an intersection we would have to be inside a voxel already
        if world.voxels.contains_key(&curr_pos)
            && let Some(step) = last_step
        {
            return Some(HitInfo::new(curr_pos, step));
        }

        let min_i = tmax.min_position();

        t = tmax[min_i];
        tmax[min_i] += tdelta[min_i];
        curr_pos[min_i] += step[min_i];

        last_step = match min_i {
            0 => Some(if step.x > 0 { HitFace::NegX } else { HitFace::PosX }),
            1 => Some(if step.y > 0 { HitFace::NegY } else { HitFace::PosY }),
            2 => Some(if step.z > 0 { HitFace::NegZ } else { HitFace::PosZ }),
            _ => unreachable!(),
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{block::BlockType, voxel::Voxel};
    use glam::Vec3;

    // Helper to create a test player at a position looking in a direction
    fn create_test_player(position: Vec3, direction: Vec3) -> Player {
        let mut player = Player::new(position);
        // Set camera direction by calculating yaw/pitch from direction vector
        let normalized = direction.normalize();
        player.camera.yaw = normalized.z.atan2(normalized.x);
        player.camera.pitch = normalized.y.asin();
        player.camera.update_vectors();
        player
    }

    // Helper to create a world with voxels at specific positions
    fn create_test_world(voxel_positions: &[IVec3]) -> World {
        let mut world = World::default();
        for &pos in voxel_positions {
            world.voxels.insert(pos, Voxel::new(pos, BlockType::Dirt));
        }
        world
    }

    #[test]
    fn test_dda_hit_single_voxel_ahead() {
        // Camera at origin looking in +X direction, voxel at (3, 0, 0)
        let player = create_test_player(Vec3::new(0.5, 0.5, 0.5), Vec3::X);
        let world = create_test_world(&[IVec3::new(3, 0, 0)]);

        let result = get_looking_at_vox_pos(&world, &player);

        assert!(result.is_some(), "Should hit voxel at (3, 0, 0)");
        let hit = result.unwrap();
        assert_eq!(hit.pos, IVec3::new(3, 0, 0));
        assert_eq!(hit.face, HitFace::NegX, "Should hit the -X face (west face)");
    }

    #[test]
    fn test_dda_no_hit_empty_world() {
        // Camera looking into empty space
        let player = create_test_player(Vec3::new(0.5, 0.5, 0.5), Vec3::X);
        let world = World::default();

        let result = get_looking_at_vox_pos(&world, &player);

        assert!(result.is_none(), "Should not hit anything in empty world");
    }

    #[test]
    fn test_dda_hit_at_exact_reach() {
        // Voxel at exactly 5.0 units away (DEFAULT_PLAYER_REACH)
        let player = create_test_player(Vec3::new(0.5, 0.5, 0.5), Vec3::X);
        let world = create_test_world(&[IVec3::new(5, 0, 0)]);

        let result = get_looking_at_vox_pos(&world, &player);

        // Should hit because we check t < REACH in the loop
        assert!(result.is_some(), "Should hit voxel within reach");
    }

    #[test]
    fn test_dda_miss_beyond_reach() {
        // Voxel at 10 units away (beyond DEFAULT_PLAYER_REACH of 5.0)
        let player = create_test_player(Vec3::new(0.5, 0.5, 0.5), Vec3::X);
        let world = create_test_world(&[IVec3::new(10, 0, 0)]);

        let result = get_looking_at_vox_pos(&world, &player);

        assert!(result.is_none(), "Should not hit voxel beyond reach");
    }

    #[test]
    fn test_dda_negative_direction() {
        // Looking in -X direction
        let player = create_test_player(Vec3::new(5.5, 0.5, 0.5), Vec3::NEG_X);
        let world = create_test_world(&[IVec3::new(2, 0, 0)]);

        let result = get_looking_at_vox_pos(&world, &player);

        assert!(result.is_some(), "Should hit voxel in negative direction");
        let hit = result.unwrap();
        assert_eq!(hit.pos, IVec3::new(2, 0, 0));
        assert_eq!(hit.face, HitFace::PosX, "Should hit the +X face (east face)");
    }

    #[test]
    fn test_dda_vertical_ray() {
        // Looking straight down
        let player = create_test_player(Vec3::new(0.5, 5.5, 0.5), Vec3::NEG_Y);
        let world = create_test_world(&[IVec3::new(0, 2, 0)]);

        let result = get_looking_at_vox_pos(&world, &player);

        assert!(result.is_some(), "Should hit voxel below");
        let hit = result.unwrap();
        assert_eq!(hit.pos, IVec3::new(0, 2, 0));
        assert_eq!(hit.face, HitFace::PosY, "Should hit the +Y face (top face)");
    }

    #[test]
    fn test_dda_diagonal_ray() {
        // Looking diagonally (normalized direction)
        let direction = Vec3::new(1.0, 0.0, 1.0).normalize();
        let player = create_test_player(Vec3::new(0.5, 0.5, 0.5), direction);
        // Place voxel on the diagonal path
        let world = create_test_world(&[IVec3::new(2, 0, 2)]);

        let result = get_looking_at_vox_pos(&world, &player);

        assert!(result.is_some(), "Should hit voxel on diagonal path");
        let hit = result.unwrap();
        assert_eq!(hit.pos, IVec3::new(2, 0, 2));
    }

    #[test]
    fn test_dda_traverse_multiple_voxels() {
        // Ray passes through empty voxels before hitting target
        let player = create_test_player(Vec3::new(0.5, 0.5, 0.5), Vec3::X);
        let world = create_test_world(&[
            IVec3::new(1, 0, 0), // First voxel in path
            IVec3::new(2, 0, 0), // Second voxel (further away)
        ]);

        let result = get_looking_at_vox_pos(&world, &player);

        // Should hit the first voxel encountered
        assert!(result.is_some());
        let hit = result.unwrap();
        assert_eq!(hit.pos, IVec3::new(1, 0, 0), "Should hit closest voxel first");
    }

    #[test]
    fn test_dda_face_detection_all_axes() {
        // Test hitting different faces

        // +X face (looking from negative side)
        let player = create_test_player(Vec3::new(-1.5, 0.5, 0.5), Vec3::X);
        let world = create_test_world(&[IVec3::new(1, 0, 0)]);
        let hit = get_looking_at_vox_pos(&world, &player).unwrap();
        assert_eq!(hit.face, HitFace::NegX);

        // +Y face (looking up from below)
        let player = create_test_player(Vec3::new(0.5, -1.5, 0.5), Vec3::Y);
        let world = create_test_world(&[IVec3::new(0, 1, 0)]);
        let hit = get_looking_at_vox_pos(&world, &player).unwrap();
        assert_eq!(hit.face, HitFace::NegY);

        // +Z face
        let player = create_test_player(Vec3::new(0.5, 0.5, -1.5), Vec3::Z);
        let world = create_test_world(&[IVec3::new(0, 0, 1)]);
        let hit = get_looking_at_vox_pos(&world, &player).unwrap();
        assert_eq!(hit.face, HitFace::NegZ);
    }

    #[test]
    fn test_dda_skip_starting_voxel() {
        // Player starts inside a voxel - should skip it and hit the next one
        let player = create_test_player(Vec3::new(0.5, 0.5, 0.5), Vec3::X);
        let world = create_test_world(&[
            IVec3::new(0, 0, 0), // Player is in this voxel
            IVec3::new(2, 0, 0), // Target voxel
        ]);

        let result = get_looking_at_vox_pos(&world, &player);

        // Should skip voxel at origin and hit voxel at (2, 0, 0)
        assert!(result.is_some());
        let hit = result.unwrap();
        assert_eq!(hit.pos, IVec3::new(2, 0, 0), "Should skip starting voxel");
    }
}
