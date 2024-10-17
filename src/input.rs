use std::collections::HashSet;

use winit;

pub struct Input {
    pressed: HashSet<winit::keyboard::Key>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
        }
    }

    pub fn is_pressed(&self, key: winit::keyboard::Key) {
        self.pressed.get(&key).is_some();
    }

    pub fn handle_key_event(&mut self, key_event: winit::event::KeyEvent) {
        match key_event.state {
            winit::event::ElementState::Pressed => {
                self.pressed.insert(key_event.logical_key);
            }
            winit::event::ElementState::Released => {
                self.pressed.remove(&key_event.logical_key);
            }
            _ => (),
        };
    }
}
