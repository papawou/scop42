#[derive(Hash, Eq, PartialEq)]
pub enum Input {
    Key(winit::keyboard::Key), // winit::event::WindowEvent::KeyboardInput;

    Mouse(winit::event::MouseButton), // coming from winit::event::WindowEvent::MouseInput;
                                      // scroll ?
                                      // HOTAS input ?
}

#[derive(Hash, Eq, PartialEq)]
pub struct DeviceInput(pub Option<winit::event::DeviceId>, pub Input);
