use std::time::Instant;

use super::traits::{self, Pressable, Releasable};

pub type Up = Input<traits::Up>;
pub type Down = Input<traits::Down>;

#[derive(Debug, Clone, Copy)]
pub struct Input<State> {
    pub at: Instant,
    pub state: std::marker::PhantomData<State>,
}
impl<State> Input<State> {
    pub fn refresh(self) -> Self {
        Self {
            at: Instant::now(),
            ..self
        }
    }

    pub fn refresh_mut(&mut self) -> &mut Self {
        self.at = Instant::now();
        self
    }
}

impl<State> Default for Input<State> {
    fn default() -> Self {
        Self {
            at: Instant::now(),
            state: std::marker::PhantomData,
        }
    }
}

impl<T: Pressable> Pressable for Input<T> {
    type Pressed = Down;

    fn press(self) -> Self::Pressed {
        Self::Pressed {
            ..Default::default()
        }
    }
}

impl<T: Releasable> Releasable for Input<T> {
    type Released = Up;

    fn release(self) -> Self::Released {
        Self::Released {
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub enum InputEnum {
    Down(Down),
    Up(Up),
}
impl InputEnum {
    pub fn as_up(&self) -> Result<&Up, &Down> {
        match self {
            Self::Down(input) => Err(input),
            Self::Up(input) => Ok(input),
        }
    }

    pub fn as_down(&self) -> Result<&Down, &Up> {
        match self {
            Self::Down(input) => Ok(input),
            Self::Up(input) => Err(input),
        }
    }

    pub fn as_up_mut(&mut self) -> Result<&mut Up, &mut Down> {
        match self {
            Self::Down(input) => Err(input),
            Self::Up(input) => Ok(input),
        }
    }

    pub fn as_down_mut(&mut self) -> Result<&mut Down, &mut Up> {
        match self {
            Self::Down(input) => Ok(input),
            Self::Up(input) => Err(input),
        }
    }

    pub fn at(&self) -> Instant {
        match self {
            Self::Down(input) => input.at,
            Self::Up(input) => input.at,
        }
    }
}
