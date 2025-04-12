use std::{collections::HashMap, time::Instant};

use winit::keyboard::KeyCode;

use super::{
    input::{Down, Input, InputEnum, Up},
    traits::Releasable,
};

pub struct InputRecorder(HashMap<KeyCode, Vec<InputEnum>>);
impl InputRecorder {
    pub fn new() -> Self {
        Self { 0: HashMap::new() }
    }

    pub fn press(&mut self, code: KeyCode, at: Instant) -> Result<&Down, &Down> {
        let mut already_in = false;

        self.0
            .entry(code)
            .and_modify(|stack| {
                match stack.last() {
                    Some(InputEnum::Down(input)) => {
                        already_in = true;
                    }
                    _ => {
                        stack.push(InputEnum::Down(Input {
                            at,
                            ..Default::default()
                        }));
                    }
                };
            })
            .or_insert(vec![InputEnum::Down(Input {
                at,
                ..Default::default()
            })]);

        let last = self
            .0
            .get(&code)
            .unwrap()
            .last()
            .unwrap()
            .as_down()
            .unwrap();

        if already_in {
            Err(last)
        } else {
            Ok(last)
        }
    }

    pub fn release(&mut self, code: KeyCode, at: Instant) -> Result<&Up, &Up> {
        let mut already_in = false;

        self.0
            .entry(code)
            .and_modify(|stack| {
                match stack.last() {
                    Some(InputEnum::Up(input)) => {
                        already_in = true;
                    }
                    _ => {
                        stack.push(InputEnum::Up(Input {
                            at,
                            ..Default::default()
                        }));
                    }
                };
            })
            .or_insert(vec![InputEnum::Up(Input {
                at,
                ..Default::default()
            })]);

        let last = self.0.get(&code).unwrap().last().unwrap().as_up().unwrap();

        if already_in {
            Err(last)
        } else {
            Ok(last)
        }
    }
}
