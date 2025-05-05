use std::{collections::HashMap, time::Instant};

use winit::keyboard::KeyCode;

use super::{
    input::{Down, Input, InputEnum, Up},
    pattern::{InputSequence, Pattern},
    traits::Releasable,
};

pub enum Err<'a> {
    ALREADY_DOWN(ALREADY_DOWN<'a>),
    ALREADY_UP(ALREADY_UP<'a>),
}
pub struct ALREADY_DOWN<'a>(&'a Down);
pub struct ALREADY_UP<'a>(&'a Up);

#[derive(Debug)]
pub struct InputRecorder(HashMap<KeyCode, Vec<InputEnum>>);
impl InputRecorder {
    pub fn new() -> Self {
        Self { 0: HashMap::new() }
    }

    pub fn press(&mut self, code: KeyCode, at: Instant) -> Result<&Down, ALREADY_DOWN> {
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
            Err(ALREADY_DOWN(last))
        } else {
            Ok(last)
        }
    }

    pub fn release(&mut self, code: KeyCode, at: Instant) -> Result<&Up, ALREADY_UP> {
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
            Err(ALREADY_UP(last))
        } else {
            Ok(last)
        }
    }
}

// convert at, to how_long
fn record_to_pattern(events: &Vec<InputEnum>) -> Pattern {
    let mut pattern = Pattern::default();

    let mut events_iter = events.iter();
    let mut left = events_iter.next();

    if let Some(left) = left {
        let mut left = left;
        for right in events_iter {
            match (left, right) {
                (&InputEnum::Down(_), &InputEnum::Up(_))
                | (&InputEnum::Up(_), &InputEnum::Down(_)) => {
                    pattern.down_mut(right.at().duration_since(left.at()));
                    left = right;
                }
                _ => {}
            }
        }
    }
    pattern
}

pub fn recorder_to_sequence(recorder: &InputRecorder) -> InputSequence {
    let mut sequence: InputSequence = InputSequence::new();

    for (key, events) in recorder.0.iter() {
        sequence.insert(*key, record_to_pattern(events));
    }
    sequence
}
