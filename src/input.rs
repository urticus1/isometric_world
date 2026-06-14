use std::cmp::PartialEq;
use std::collections::HashMap;
use minifb::Key;
use crate::input::ButtonState::{Held, Pressed};

pub struct InputBuffer {
    button_states: HashMap<Key, ButtonState>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ButtonState {
    Pressed,
    Released,
    Held,
}


impl InputBuffer {
    pub fn new() -> InputBuffer {
        InputBuffer {
            button_states: HashMap::new()
        }
    }

    pub fn update_button_states(&mut self, keys_down: Vec<Key>) {
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
    }

    pub fn button_pressed(&self, key: Key) -> bool {
        match self.button_states.get(&key) {
            Some(ButtonState::Pressed) => true,
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