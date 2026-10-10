use glam::Vec3;

use crate::{
    engine::world::World,
    input::KeyState,
    physics::{GRAVITY, PhysicsBody},
    player::{DEFAULT_PLAYER_JUMP_HEIGHT, DEFAULT_PLAYER_SPEED},
    render::camera::Camera,
};

pub trait MovementBackend {
    fn tick(&self, body: &mut PhysicsBody, world: &World, up: KeyState, dt: f32);
    fn velocity_from_input(&self, input_velocity: Vec3, camera: &Camera, speed: f32) -> Vec3;
}

#[derive(Debug)]
pub enum Movement {
    Walk(Walk),
    Fly(Fly),
}

impl<'a> AsRef<dyn MovementBackend + 'a> for Movement {
    fn as_ref(&self) -> &(dyn MovementBackend + 'a) {
        match self {
            Self::Walk(walk) => walk as &dyn MovementBackend,
            Self::Fly(fly) => fly as &dyn MovementBackend,
        }
    }
}

impl Movement {
    pub fn walk() -> Self {
        Self::Walk(Walk)
    }

    pub fn fly() -> Self {
        Movement::Fly(Fly)
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Fly;

impl MovementBackend for Fly {
    fn velocity_from_input(&self, input_velocity: Vec3, camera: &Camera, speed: f32) -> Vec3 {
        let input_vel_transformed = (camera.front * input_velocity.x)
            + (camera.right * input_velocity.z)
            + (camera.up * input_velocity.y);
        input_vel_transformed * speed
    }

    fn tick(&self, body: &mut PhysicsBody, _world: &World, up: KeyState, dt: f32) {
        if up.is_pressed {
            body.velocity.y = DEFAULT_PLAYER_SPEED;
        }
        body.step(dt);
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Walk;

impl MovementBackend for Walk {
    fn velocity_from_input(&self, input_velocity: Vec3, camera: &Camera, speed: f32) -> Vec3 {
        let input_vel_transformed =
            (camera.front * input_velocity.x) + (camera.right * input_velocity.z);
        input_vel_transformed * speed
    }

    fn tick(&self, body: &mut PhysicsBody, world: &World, up: KeyState, dt: f32) {
        let is_colliding = world.is_colliding(body);
        if !is_colliding {
            body.velocity.y += GRAVITY * dt;
        } else {
            body.velocity.y = 0.;
        }

        if up.just_pressed && is_colliding {
            body.velocity.y = get_initial_jump_vel(DEFAULT_PLAYER_JUMP_HEIGHT);
        }

        body.step(dt);
    }
}

/// Get the initial velocity of a jump that will reach height `h`
fn get_initial_jump_vel(h: f32) -> f32 {
    assert!(h >= 0., "Jump height must be >= 0");
    (2. * h * GRAVITY.abs()).sqrt()
}
