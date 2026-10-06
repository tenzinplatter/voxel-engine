use beryllium::events::*;
use glam::Vec3;
use tracing::warn;

pub enum MovementKeys {
    Forward,
    Back,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Default, Debug)]
pub struct KeyState {
    pub is_pressed: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}

impl KeyState {
    pub fn end_frame(&mut self) {
        self.just_pressed = false;
        self.just_released = false;
    }

    /// Creates a key state by comparing previous and current press states.
    fn from_pressed_last_and_curr(last_pressed: bool, curr_pressed: bool) -> Self {
        Self {
            is_pressed: curr_pressed,
            just_pressed: !last_pressed && curr_pressed,
            just_released: last_pressed && !curr_pressed,
        }
    }

    fn update(&mut self, pressed: bool) {
        *self = dbg!(Self::from_pressed_last_and_curr(self.is_pressed, pressed));
    }
}

/// Represents the key events that occurred in a single frame
#[derive(Default, Debug)]
pub struct InputState {
    pub forward: KeyState,
    pub back: KeyState,
    pub left: KeyState,
    pub right: KeyState,
    pub up: KeyState,
    pub mb1: KeyState,
    pub mb3: KeyState,
    pub number_keys: [KeyState; 10],
    pub movement_toggle: KeyState,
}

impl InputState {
    fn keys_mut(&mut self) -> impl Iterator<Item = &mut KeyState> {
        let Self {
            forward,
            back,
            left,
            right,
            up,
            mb1,
            mb3,
            number_keys,
            movement_toggle,
        } = self;

        [forward, back, left, right, up, mb1, mb3, movement_toggle]
            .into_iter()
            .chain(number_keys)
    }

    pub fn reset(&mut self) {
        self.keys_mut().for_each(KeyState::end_frame);
    }

    /// Converts the current input state to a normalized velocity vector.
    pub fn as_vel(&self) -> Vec3 {
        fn axis(positive: &KeyState, negative: &KeyState) -> f32 {
            f32::from(positive.is_pressed) - f32::from(negative.is_pressed)
        }

        let vel = Vec3::new(
            axis(&self.forward, &self.back),
            f32::from(self.up.is_pressed),
            axis(&self.right, &self.left),
        );

        vel.try_normalize().unwrap_or(vel)
    }

    /// Updates the state when a key is pressed or released.
    pub fn set_key(&mut self, keycode: SDL_Keycode, pressed: bool) {
        #[allow(non_upper_case_globals)]
        let key = match keycode {
            SDLK_w => &mut self.forward,
            SDLK_s => &mut self.back,
            SDLK_a => &mut self.left,
            SDLK_d => &mut self.right,
            SDLK_m => &mut self.movement_toggle,
            SDLK_SPACE => &mut self.up,
            SDLK_1 => &mut self.number_keys[0],
            SDLK_2 => &mut self.number_keys[1],
            _ => {
                warn!(?keycode, "unmapped key press");
                return;
            }
        };

        key.update(pressed);
    }

    pub fn set_mouse_button(&mut self, button: u8, pressed: bool) {
        match button {
            1 => self.mb1 = KeyState::from_pressed_last_and_curr(self.mb1.is_pressed, pressed),
            3 => self.mb3 = KeyState::from_pressed_last_and_curr(self.mb3.is_pressed, pressed),
            _ => {}
        }
    }

    pub fn reset_keys(&mut self) {
        self.forward.just_pressed = false;
        self.back.just_pressed = false;
        self.left.just_pressed = false;
        self.right.just_pressed = false;
        self.up.just_pressed = false;
        for key_state in &mut self.number_keys {
            key_state.just_pressed = false;
        }
    }

    pub fn reset_mouse_buttons(&mut self) {
        // TODO: possibly need to reset just_released as well?
        // also not even sure why we need to reset these
        self.mb1.just_pressed = false;
        self.mb3.just_pressed = false;
    }

    /// gets the state of a number key (0-9), where 1 maps to index 0, and 0 maps to index 9
    pub fn number_key(&self, number: usize) -> &KeyState {
        if number == 0 {
            return &self.number_keys[9];
        }

        &self.number_keys[number - 1]
    }
}
