use winit::keyboard::KeyCode;

use crate::input::input::InputEnum;

pub type Queue = Vec<(KeyCode, InputEnum)>;
