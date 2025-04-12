use std::{
    collections::{hash_map, HashMap, HashSet, VecDeque},
    default,
    hash::Hash,
    time::{Duration, Instant},
};

use input::{Down, Input, Up};
use pattern::InputSequence;
use traits::{Pressable, Releasable};
use winit::keyboard::{Key, KeyCode};

pub mod input;
pub mod pattern;
pub mod recorder;
pub mod traits;
