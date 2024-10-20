struct MouseEvent {
    device_id: winit::event::DeviceId,
    state: winit::event::ElementState,
    button: winit::event::MouseButton,
}

pub enum InputType {
    Key(winit::keyboard::PhysicalKey),
    Mouse(winit::event::MouseButton),
}

pub enum InputEvent {
    Key(winit::event::KeyEvent),
    Mouse(MouseEvent),
}
