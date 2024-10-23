use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    time::Instant,
};

pub enum State {
    Pressed(Instant),
    Released(Instant),
}

pub struct InputManager {
    keys: HashMap<DeviceInput, State>,
    queue: VecDeque<Event>,
}
impl InputManager {
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
}

#[derive(Hash, Eq, PartialEq)]
pub enum Input {
    Key(winit::keyboard::Key), // winit::event::WindowEvent::KeyboardInput;

    Mouse(winit::event::MouseButton), // coming from winit::event::WindowEvent::MouseInput;
                                      // scroll ?
                                      // HOTAS input ?
}

#[derive(Hash, Eq, PartialEq)]
pub struct DeviceInput(pub Option<winit::event::DeviceId>, pub Input);

/**
 * Event
*/

pub enum Event {
    Mouse(MouseEvent),
    Key(KeyEvent),
}

// eq. winit::window::KeyboardInput
impl TryInto<Event> for winit::event::WindowEvent {
    type Error = (); // todo! ?
    fn try_into(self) -> Result<Event, Self::Error> {
        match self {
            winit::event::WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => Ok(Event::Mouse(MouseEvent {
                device_id,
                state,
                button,
            })),
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => Ok(Event::Key(KeyEvent { device_id, event })),
            _ => Err(()),
        }
    }
}

pub trait EventHandler<E> {
    fn handle_event(&mut self, event: E);
}

impl EventHandler<Event> for InputManager {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Mouse(mouse_event) => self.handle_event(mouse_event),
            Event::Key(key_event) => self.handle_event(key_event),
        }
    }
}

// eq. winit::window::MouseInput
struct KeyEvent {
    pub device_id: winit::event::DeviceId,
    pub event: winit::event::KeyEvent,
}
impl EventHandler<KeyEvent> for InputManager {
    fn handle_event(&mut self, key_event: KeyEvent) {
        let device_input = DeviceInput(
            Some(key_event.device_id),
            Input::Key(key_event.event.logical_key.clone()),
        );

        match key_event.event.state {
            winit::event::ElementState::Pressed => {
                self.press(device_input);
            }
            winit::event::ElementState::Released => {
                self.release(device_input);
            }
        };

        self.queue.push_back(Event::Key(key_event));
    }
}

struct MouseEvent {
    pub device_id: winit::event::DeviceId,
    pub state: winit::event::ElementState,
    pub button: winit::event::MouseButton,
}
impl EventHandler<MouseEvent> for InputManager {
    fn handle_event(&mut self, mouse_event: MouseEvent) {
        let device_input = DeviceInput(
            Some(mouse_event.device_id),
            Input::Mouse(mouse_event.button),
        );

        match mouse_event.state {
            winit::event::ElementState::Pressed => {
                self.press(device_input);
            }
            winit::event::ElementState::Released => {
                self.release(device_input);
            }
        };

        self.queue.push_back(Event::Mouse(mouse_event));
    }
}
