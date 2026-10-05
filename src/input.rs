use beryllium::events::*;
use glam::Vec3;

#[derive(Default, Debug)]
pub struct KeyState {
    pub is_pressed: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}

pub enum MovementKeys {
    Forward,
    Back,
    Left,
    Right,
    Up,
    Down,
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

impl KeyState {
    /// Creates a key state by comparing previous and current press states.
    fn from_pressed_last_and_curr(last_pressed: bool, curr_pressed: bool) -> Self {
        Self {
            is_pressed: curr_pressed,
            just_pressed: !last_pressed && curr_pressed,
            just_released: last_pressed && !curr_pressed,
        }
    }
}

impl InputState {
    /// Converts the current input state to a normalized velocity vector.
    pub fn as_vel(&self) -> Vec3 {
        let mut accel = Vec3::ZERO;

        if self.forward.is_pressed {
            accel.x += 1.;
        }

        if self.back.is_pressed {
            accel.x -= 1.;
        }

        if self.left.is_pressed {
            accel.z -= 1.;
        }

        if self.right.is_pressed {
            accel.z += 1.;
        }

        if self.up.is_pressed {
            accel.y += 1.;
        }

        // Normalize to prevent faster diagonal movement
        if accel.length_squared() > 0.0 {
            accel = accel.normalize();
        }

        accel
    }

    /// Updates the state when a key is pressed or released.
    pub fn set_key(&mut self, keycode: SDL_Keycode, pressed: bool) {
        // TODO: make this a macro
        #[allow(non_upper_case_globals)]
        match keycode {
            SDLK_w => self.forward = KeyState::from_pressed_last_and_curr(self.forward.is_pressed, pressed),
            SDLK_s => self.back = KeyState::from_pressed_last_and_curr(self.back.is_pressed, pressed),
            SDLK_a => self.left = KeyState::from_pressed_last_and_curr(self.left.is_pressed, pressed),
            SDLK_d => self.right = KeyState::from_pressed_last_and_curr(self.right.is_pressed, pressed),
            SDLK_m => self.movement_toggle = KeyState::from_pressed_last_and_curr(self.movement_toggle.is_pressed, pressed),
            SDLK_SPACE => self.up = KeyState::from_pressed_last_and_curr(self.up.is_pressed, pressed),
            SDLK_1 => self.number_keys[0] = KeyState::from_pressed_last_and_curr(self.number_keys[0].is_pressed, pressed),
            SDLK_2 => self.number_keys[1] = KeyState::from_pressed_last_and_curr(self.number_keys[1].is_pressed, pressed),
            _ => {}
        }
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
