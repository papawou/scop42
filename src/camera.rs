use glam::Vec3;

use crate::input::{self};

pub struct Camera {
    pub position: Vec3,
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }
}

pub trait CameraControls {
    fn move_forward(&mut self, distance: f32);
    fn move_backward(&mut self, distance: f32);
    fn move_left(&mut self, distance: f32);
    fn move_right(&mut self, distance: f32);
}

impl CameraControls for Camera {
    fn move_forward(&mut self, distance: f32) {
        self.position += Vec3::Z * distance;
    }

    fn move_backward(&mut self, distance: f32) {
        self.position -= Vec3::Z * distance;
    }

    fn move_left(&mut self, distance: f32) {
        self.position -= Vec3::X * distance;
    }

    fn move_right(&mut self, distance: f32) {
        self.position += Vec3::X * distance;
    }
}

impl Camera {
    fn process_input(&mut self, input: &input::winit_impl::WinitInputManager) {
        if input.is_press(winit::keyboard::KeyCode::KeyW).is_some() {
            self.move_forward(0.1);
        }
        if input
            .is_press(winit::keyboard::Key::Character("S"))
            .is_some()
        {
            self.move_backward(0.1);
        }
        if input
            .is_press(winit::keyboard::Key::Character("A"))
            .is_some()
        {
            self.move_left(0.1);
        }
        if input
            .is_press(winit::keyboard::Key::Character("D"))
            .is_some()
        {
            self.move_right(0.1);
        }
    }
}
