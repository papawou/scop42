use std::{
    collections::{hash_map, HashMap, HashSet, VecDeque},
    default,
    hash::Hash,
    time::{Duration, Instant},
};

use button::{Button, Down, Up};
use input::Input;
use pattern::InputSequence;
use traits::{Pressable, Releasable};
use winit::keyboard::{Key, KeyCode};

pub mod button;
pub mod input;
pub mod pattern;
pub mod traits;
pub mod winit_impl;

pub enum InputEnum {
    Down(Input<Down>),
    Up(Input<Up>),
}

pub struct BAD_STATE<'a, State>(&'a Input<State>);

impl InputEnum {
    pub fn press(&mut self) -> &Input<Down> {
        match self {
            Self::Down(input) => input.refresh(),
            Self::Up(input) => {
                *self = Self::Down(input.press());
                match self {
                    Self::Down(input) => input,
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn release(&mut self) -> &Input<Up> {
        match self {
            Self::Up(input) => input.refresh(),
            Self::Down(input) => {
                *self = Self::Up(input.release());
                match self {
                    Self::Up(input) => input,
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn try_press(&mut self) -> Result<&Input<Down>, BAD_STATE<Down>> {
        match self {
            Self::Down(input) => Err(BAD_STATE(input)),
            Self::Up(input) => {
                *self = Self::Down(input.press());
                match self {
                    Self::Down(input) => Ok(input),
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn try_release(&mut self) -> Result<&Input<Up>, BAD_STATE<Up>> {
        match self {
            Self::Down(input) => {
                *self = Self::Up(input.release());
                match self {
                    Self::Up(input) => Ok(input),
                    _ => unreachable!(),
                }
            }
            Self::Up(input) => Err(BAD_STATE(input)),
        }
    }

    pub fn at(&self) -> Instant {
        match self {
            Self::Down(input) => input.at,
            Self::Up(input) => input.at,
        }
    }
}

pub type Manager = HashMap<KeyCode, InputEnum>;
