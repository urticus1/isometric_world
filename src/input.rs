use std::cmp::PartialEq;
use std::collections::HashMap;
use minifb::{Key, MouseButton};
use crate::input::ButtonState::{Held, Pressed, Released};

pub struct InputBuffer {
    button_states: HashMap<Key, ButtonState>,
    left_mouse_state: ButtonState,
    right_mouse_state: ButtonState,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ButtonState {
    Pressed,
    Released,
    Held,
}

pub struct InputState {
    pub selected_agent: Option<u8>,
}

impl InputBuffer {
    pub fn new() -> InputBuffer {
        InputBuffer {
            button_states: HashMap::new(),
            left_mouse_state: Released,
            right_mouse_state: Released,
        }
    }

    pub fn update_button_states(&mut self, keys_down: Vec<Key>, left_mouse_down: bool, right_mouse_down: bool) {
        let existing_keys: Vec<Key> = self.button_states.keys().copied().collect();
        for key in  existing_keys {
            if !keys_down.contains(&key) {
                if let Some(state) = self.button_states.get(&key) {
                    if *state == ButtonState::Released {
                        self.button_states.remove(&key);
                    }
                    else {
                        self.button_states.insert(key, ButtonState::Released);
                    }
                }
            }
        }
        for key in keys_down {
            if self.button_states.contains_key(&key) {
                self.button_states.insert(key, Held);
            }
            else {
                self.button_states.insert(key, Pressed);
            }
        }

        if left_mouse_down {
            if self.left_mouse_state == Pressed {
                self.left_mouse_state = ButtonState::Held;
            }
            else {
                self.left_mouse_state = ButtonState::Pressed;
            }
        }
        else {
            self.left_mouse_state = ButtonState::Released;
        }
    }

    pub fn left_mouse_pressed(&self) -> bool {
        match self.left_mouse_state {
            ButtonState::Pressed => true,
            _ => false
        }
    }

    pub fn right_mouse_pressed(&self) -> bool {
        match self.right_mouse_state {
            ButtonState::Pressed => true,
            _ => false
        }
    }

    pub fn button_pressed(&self, key: Key) -> bool {
        match self.button_states.get(&key) {
            Some(ButtonState::Pressed) => true,
            _ => false
        }
    }

    pub fn button_pressed_or_held(&self, key: Key) -> bool {
        match self.button_states.get(&key) {
            Some(ButtonState::Pressed) => true,
            Some(ButtonState::Held) => true,
            _ => false
        }
    }

    pub fn button_released(&self, key: Key) -> bool {
        match self.button_states.get(&key) {
            Some(ButtonState::Released) => true,
            _ => false
        }
    }

    pub fn button_held(&self, key: Key) -> bool {
        match self.button_states.get(&key) {
            Some(ButtonState::Held) => true,
            _ => false
        }
    }
}