use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    time::Instant,
};

use event::Event;
use input::{DeviceInput, Input};
mod event;
mod input;

pub enum State {
    Pressed(Instant),
    Released(Instant),
}

pub struct WinitInputManager {
    keys: HashMap<DeviceInput, State>,
    queue: VecDeque<Event>,
}
impl WinitInputManager {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            queue: VecDeque::new(),
        }
    }

    fn press(&mut self, input: DeviceInput) {
        self.keys
            .insert(input, State::Pressed(std::time::Instant::now()));
    }

    fn release(&mut self, input: DeviceInput) {
        self.keys
            .insert(input, State::Released(std::time::Instant::now()));
    }

    fn hold(&mut self, input: DeviceInput) {
        todo!()
    }

    fn is_press(&mut self, input: &DeviceInput) -> bool {
        match self.keys.get(&input) {
            Some(State::Pressed(_)) => true,
            _ => false,
        }
    }

    fn is_release(&mut self, input: &DeviceInput) -> bool {
        match self.keys.get(&input) {
            Some(State::Released(_)) => true,
            _ => false,
        }
    }

    fn push_event(&mut self, event: Event) {
        match &event {
            Event::Key(key_event) => {
                let device_id = DeviceInput(
                    Some(key_event.device_id),
                    Input::Key(key_event.event.logical_key.clone()),
                );
                match key_event.event.state {
                    winit::event::ElementState::Pressed => {
                        self.press(device_id);
                    }
                    winit::event::ElementState::Released => {
                        self.release(device_id);
                    }
                }
            }
            Event::Mouse(mouse_event) => {
                let device_id = DeviceInput(
                    Some(mouse_event.device_id),
                    Input::Mouse(mouse_event.button),
                );
                match mouse_event.state {
                    winit::event::ElementState::Pressed => {
                        self.press(device_id);
                    }
                    winit::event::ElementState::Released => {
                        self.release(device_id);
                    }
                }
            }
        };

        self.queue.push_back(event);
    }
}
