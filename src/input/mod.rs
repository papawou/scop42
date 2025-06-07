use std::{
    collections::{hash_map, HashMap, HashSet, VecDeque},
    default,
    hash::Hash,
    time::{Duration, Instant},
};

use input::{Down, Input, Up};
use traits::{Pressable, Releasable};
use winit::keyboard::{Key, KeyCode};

use crate::input::{
    input::InputEnum,
    pattern::{Event, Pattern, State},
    queue::Queue,
    recorder::InputRecorder,
    sequence::InputSequence,
};

pub mod input;
pub mod pattern;
pub mod queue;
pub mod recorder;
pub mod sequence;
pub mod traits;

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

/**
 * clean pattern: trim VOID, merge consecutive State
 */
pub fn record_match(events: &Vec<InputEnum>, pattern: &Pattern) -> bool {
    let base_instant = Instant::now();
    for (i, _) in events.iter().enumerate() {
        let mut events_it = events[i..].iter().peekable();
        let mut pattern_it = pattern.events.iter().peekable();

        while let (Some(pat), Some(evt)) = (pattern_it.next(), events_it.next()) {
            let right_evt = events_it.peek();
            let right_pat = pattern_it.peek();

            let duration_e: Option<Duration> = {
                match events_it.peek() {
                    None => None, // last right_e
                    Some(right_evt) => Some(right_evt.at().duration_since(evt.at())),
                }
            };
        }

        if pattern_it.next().is_none() {
            return true;
        }
    }

    false
}

pub fn test(sequence: &Vec<InputEnum>, pattern: &Pattern) -> bool {
    struct Candidate {
        at_range: (Instant, Instant),
        last_index: usize, // last index matching pattern
    }
    let candidates: Vec<Candidate> = vec![];
    let mut sequence = sequence.iter().peekable();

    while let Some(event) = sequence.next() {
        let Some(next_event) = sequence.peek() else {
            continue;
        };
        let mut new_candidates: Vec<Candidate> = vec![];

        for candidate in &candidates {
            match (event, pattern.events.get(candidate.last_index + 1)) {
                (
                    InputEnum::Down(_),
                    Some(Event {
                        state: State::DOWN,
                        duration: duration_pattern,
                    }),
                )
                | (
                    InputEnum::Up(_),
                    Some(Event {
                        state: State::UP,
                        duration: duration_pattern,
                    }),
                ) => {
                    if event.at() < candidate.at_range.0 && candidate.at_range.1 < event.at() {
                        // event at_start doesn't fit in candidate.at_range
                    }

                    new_candidates.push(Candidate {
                        at_range: (event.at(), event.at()),
                        last_index: candidate.last_index + 1,
                    })
                }
                (_, None) => {
                    return true;
                }
                _ => {}
            }
        }
    }

    false
}

pub fn recorder_to_queue(recorder: &InputRecorder) -> Queue {
    let mut queue: Queue = recorder
        .0
        .iter()
        .flat_map(|(key, events)| events.iter().map(|event| (*key, event.clone())))
        .collect();

    queue.sort_by_key(|(_, input)| input.at());
    queue
}
