use std::time::Instant;

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
    fn is_press(&mut self, input: WinitDeviceInput) -> bool {
        match self.keys.get(&input) {
            Some(State::Pressed(_)) => true,
            _ => false,
        }
    }

    fn is_release(&mut self, input: &WinitDeviceInput) -> bool {
        match self.keys.get(&input) {
            Some(State::Released(_)) => true,
            _ => false,
        }
    }

    fn press(&mut self, input: &WinitDeviceInput) -> bool {
        dbg!("{:?}", input);
        self.keys
            .insert(input.clone(), State::Pressed(Instant::now()))
            .is_none()
    }

    fn release(&mut self, input: &WinitDeviceInput) -> bool {
        self.keys
            .insert(input.clone(), State::Released(Instant::now()))
            .is_none()
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
                        self.press(&device_input);
                    }
                    winit::event::ElementState::Released => {
                        self.release(&device_input);
                    }
                },
                _ => {}
            }
        }
    }
}
