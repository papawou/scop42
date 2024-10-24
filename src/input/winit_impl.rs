use std::{collections::hash_map, time::Instant};

use super::{DeviceInput, InputManager, Manager, State};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Input {
    Key(winit::keyboard::Key), // winit::event::WindowEvent::KeyboardInput;

    Mouse(winit::event::MouseButton), // coming from winit::event::WindowEvent::MouseInput;
                                      // scroll ?
                                      // HOTAS input ?
}

pub type WinitDeviceInput = DeviceInput<Option<winit::event::DeviceId>, Input>;
pub type WinitInputManager = InputManager<WinitDeviceInput>;

impl Manager<WinitDeviceInput> for WinitInputManager {
    fn is_press(&mut self, input: WinitDeviceInput) -> Option<Instant> {
        self.keys.get(&input).and_then(|state| match state {
            State::Pressed(instant) => Some(instant.clone()),
            _ => None,
        })
    }

    fn is_release(&mut self, input: WinitDeviceInput) -> Option<Instant> {
        self.keys.get(&input).and_then(|state| match state {
            State::Released(instant) => Some(instant.clone()),
            _ => None,
        })
    }

    // dont refresh press timer
    fn try_press(&mut self, input: WinitDeviceInput) -> Option<State> {
        match self.keys.entry(input.clone()) {
            hash_map::Entry::Occupied(o) => match o.get() {
                State::Pressed(instant) => Some(State::Pressed(*instant)),
                _ => self.press(input.clone()),
            },
            hash_map::Entry::Vacant(v) => self.press(input.clone()),
        }
    }

    // dont refresh release timer
    fn try_release(&mut self, input: WinitDeviceInput) -> Option<State> {
        match self.keys.entry(input.clone()) {
            hash_map::Entry::Occupied(o) => match o.get() {
                State::Released(instant) => Some(State::Released(*instant)),
                _ => self.release(input.clone()),
            },
            hash_map::Entry::Vacant(v) => self.release(input.clone()),
        }
    }

    fn press(&mut self, input: WinitDeviceInput) -> Option<State> {
        self.keys.insert(input, State::Pressed(Instant::now()))
    }

    fn release(&mut self, input: WinitDeviceInput) -> Option<State> {
        self.keys.insert(input, State::Released(Instant::now()))
    }
}

impl TryFrom<winit::event::WindowEvent> for WinitDeviceInput {
    type Error = ();
    fn try_from(value: winit::event::WindowEvent) -> Result<Self, Self::Error> {
        match value {
            winit::event::WindowEvent::KeyboardInput {
                device_id, event, ..
            } => Ok(DeviceInput(Some(device_id), Input::Key(event.logical_key))),
            winit::event::WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => Ok(DeviceInput(Some(device_id), Input::Mouse(button))),
            _ => Err(()),
        }
    }
}

impl WinitInputManager {
    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        if let Ok(device_input) = event.clone().try_into() {
            match event {
                // keyboard input
                winit::event::WindowEvent::KeyboardInput {
                    event: winit::event::KeyEvent { state, .. },
                    ..
                }
                // mouse input
                | winit::event::WindowEvent::MouseInput { state, .. } =>
                // process input
                match state {
                    winit::event::ElementState::Pressed => {
                        self.try_press(device_input);
                    }
                    winit::event::ElementState::Released => {
                        self.try_release(device_input);
                    }
                },
                _ => {}
            }
        }
    }
}
