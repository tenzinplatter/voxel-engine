use std::collections::HashMap;

use beryllium::{
    Sdl,
    events::{self, *},
};
use glam::Vec3;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum InputKey {
    MousePress(u8),
    KeyPress(SDL_Keycode),
}

impl From<SDL_Keycode> for InputKey {
    fn from(value: SDL_Keycode) -> Self {
        InputKey::KeyPress(value)
    }
}

impl From<u8> for InputKey {
    fn from(value: u8) -> Self {
        InputKey::MousePress(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Forward,
    Back,
    Left,
    Right,
    Up,
    Down,
    PlaceBlock,
    DestroyBlock,
    ToggleMovement,
    Block1,
    Block2,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct MouseDelta {
    pub x: i32,
    pub y: i32,
}

/// Represents the key events that occurred in a single frame
#[derive(Debug, Default)]
pub struct InputState {
    input_keys: HashMap<InputKey, Key>,
    key_states: HashMap<Key, KeyState>,
    mouse_delta: MouseDelta,
    received_quit: bool,
}

impl InputState {
    pub fn should_quit(&self) -> bool {
        self.received_quit
    }

    pub fn mouse_delta(&self) -> MouseDelta {
        self.mouse_delta
    }

    pub fn register(&mut self, key: Key, keycode: InputKey) {
        self.input_keys.insert(keycode, key);
        self.key_states.insert(key, KeyState::default());
    }

    pub fn get(&self, key: Key) -> KeyState {
        self.key_states.get(&key).copied().unwrap_or_default()
    }

    pub fn just_pressed(&self, key: Key) -> bool {
        self.get(key).just_pressed
    }

    pub fn is_pressed(&self, key: Key) -> bool {
        self.get(key).is_pressed
    }

    pub fn just_released(&self, key: Key) -> bool {
        self.get(key).just_released
    }

    /// Converts the current input state to a normalized velocity vector.
    pub fn as_vel(&self) -> Vec3 {
        fn axis(positive: KeyState, negative: KeyState) -> f32 {
            f32::from(positive.is_pressed) - f32::from(negative.is_pressed)
        }

        let vel = Vec3::new(
            axis(self.get(Key::Forward), self.get(Key::Back)),
            f32::from(self.get(Key::Up).is_pressed),
            axis(self.get(Key::Right), self.get(Key::Left)),
        );

        vel.try_normalize().unwrap_or(vel)
    }

    pub fn end_frame(&mut self) {
        self.key_states.values_mut().for_each(KeyState::end_frame);
        self.mouse_delta = MouseDelta::default();
    }

    /// Updates the state when a key is pressed or released.
    fn set_input(&mut self, keycode: InputKey, pressed: bool) {
        if let Some(key) = self.input_keys.get(&keycode)
            && let Some(state) = self.key_states.get_mut(key)
        {
            state.update(pressed);
        }
    }

    pub fn register_defaults(&mut self) {
        self.register(Key::Forward, SDLK_w.into());
        self.register(Key::Back, SDLK_s.into());
        self.register(Key::Left, SDLK_a.into());
        self.register(Key::Right, SDLK_d.into());
        self.register(Key::Up, SDLK_SPACE.into());
        self.register(Key::Down, SDLK_LCTRL.into());
        self.register(Key::ToggleMovement, SDLK_m.into());
        self.register(Key::Block1, SDLK_1.into());
        self.register(Key::Block2, SDLK_2.into());
        self.register(Key::PlaceBlock, InputKey::MousePress(3));
        self.register(Key::DestroyBlock, InputKey::MousePress(1));
    }

    /// Poll the provided sdl instance for input events
    pub fn poll(&mut self, sdl: &Sdl) {
        while let Some(event) = sdl.poll_events() {
            match event {
                (events::Event::Quit, _) => self.received_quit = true,
                (
                    events::Event::Key {
                        keycode, pressed, ..
                    },
                    _,
                ) => {
                    self.set_input(keycode.into(), pressed);
                }
                (
                    events::Event::MouseMotion {
                        x_delta, y_delta, ..
                    },
                    _,
                ) => {
                    self.mouse_delta.x += x_delta;
                    self.mouse_delta.y += y_delta;
                }
                (
                    events::Event::MouseButton {
                        button, pressed, ..
                    },
                    _,
                ) => {
                    self.set_input(button.into(), pressed);
                }
                _ => {}
            }
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
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
        *self = Self::from_pressed_last_and_curr(self.is_pressed, pressed);
    }
}
