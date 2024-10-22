use std::collections::{HashMap, VecDeque};

use super::{InputHandler, InputManager, State};

// EVENTS
// winit::event::WindowEvent::MouseInput
struct MouseEvent {
    device_id: winit::event::DeviceId,
    state: winit::event::ElementState,
    button: winit::event::MouseButton,
}
type KeyEvent = winit::event::KeyEvent;

pub enum InputEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
}

// INPUT
type KeyInput = winit::keyboard::Key;
struct MouseInput = winit::event::MouseButton;

#[derive(Hash, Eq, PartialEq)]
pub enum Input {
    KeyInput,
    MouseInput, // scroll ?
}

// WINIT
pub struct WinitInputManager {
    keys: HashMap<Input, State>,
    queue: VecDeque<InputEvent>,
}

impl InputHandler<Input> for WinitInputManager
where
    Input: Eq + std::hash::Hash,
{
    fn press(&mut self, input: Input) {
        self.keys
            .insert(input, State::Pressed(std::time::Instant::now()));
    }

    fn release(&mut self, input: Input) {
        self.keys
            .insert(input, State::Released(std::time::Instant::now()));
    }

    fn hold(&mut self, input: Input) {
        self.keys
            .insert(input, State::Pressed(std::time::Instant::now()));
    }

    fn is_press(&mut self, input: &Input) -> bool {
        match self.keys.get(input) {
            Some(State::Pressed(_)) => true,
            _ => false,
        }
    }

    fn is_release(&mut self, input: &Input) -> bool {
        match self.keys.get(input) {
            Some(State::Released(_)) => true,
            _ => false,
        }
    }
}
