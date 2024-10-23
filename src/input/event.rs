pub enum Event {
    Mouse(MouseEvent),
    Key(KeyEvent),
}

// eq. winit::window::KeyboardInput
pub struct MouseEvent {
    pub device_id: winit::event::DeviceId,
    pub state: winit::event::ElementState,
    pub button: winit::event::MouseButton,
}

// eq. winit::window::MouseInput
pub struct KeyEvent {
    pub device_id: winit::event::DeviceId,
    pub event: winit::event::KeyEvent,
}
