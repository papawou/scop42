use std::collections::HashMap;

use winit::keyboard::KeyCode;

use crate::input::pattern::Pattern;

pub type InputSequence = HashMap<KeyCode, Pattern>;
