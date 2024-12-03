use std::{
    collections::{hash_map, HashMap, HashSet, VecDeque},
    hash::Hash,
    time::Instant,
};

use button::{Button, Down, Up};
use input::Input;
use traits::{Pressable, Releasable};
use winit::keyboard::KeyCode;

pub mod button;
pub mod input;
pub mod traits;
pub mod winit_impl;

// Button -> Input -> InputEnum
type Manager = HashMap<KeyCode, InputEnum>;

enum InputEnum {
    Down(Input<Down>),
    Up(Input<Up>),
}
