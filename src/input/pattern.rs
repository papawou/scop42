use std::{
    collections::HashMap,
    ops::Add,
    time::{Duration, Instant},
};

use anyhow::bail;
use winit::keyboard::KeyCode;

use super::input::InputEnum;

#[derive(Debug, PartialEq)]
pub enum State {
    UP,   // +
    DOWN, // -
    VOID, // o -- no impact
}

// A +---ooo+-+
// B ++---oo--

#[derive(Debug)]
pub struct Event {
    state: State,
    duration: Duration,
}
impl Event {
    pub fn new() -> Self {
        Event {
            state: State::DOWN,
            duration: Duration::ZERO,
        }
    }

    pub fn duration(self, duration: Duration) -> Self {
        Self { duration, ..self }
    }

    pub fn down(self, duration: Duration) -> Self {
        Self {
            state: State::DOWN,
            duration,
        }
    }

    pub fn up(self, duration: Duration) -> Self {
        Self {
            state: State::UP,
            duration,
        }
    }

    pub fn void(self, duration: Duration) -> Self {
        Self {
            state: State::VOID,
            duration,
        }
    }

    pub fn state(self, state: State) -> Self {
        Self { state, ..self }
    }

    pub fn duration_mut(&mut self, duration: Duration) -> &mut Self {
        self.duration = duration;
        self
    }

    pub fn down_mut(&mut self, duration: Duration) -> &mut Self {
        self.state = State::DOWN;
        self.duration = duration;
        self
    }

    pub fn up_mut(&mut self, duration: Duration) -> &mut Self {
        self.state = State::UP;
        self.duration = duration;
        self
    }

    pub fn void_mut(&mut self, duration: Duration) -> &mut Self {
        self.state = State::VOID;
        self.duration = duration;
        self
    }

    pub fn state_mut(&mut self, state: State) -> &mut Self {
        self.state = state;
        self
    }
}

#[derive(Debug)]
pub struct Pattern {
    events: Vec<Event>,
}
impl Default for Pattern {
    fn default() -> Self {
        Self { events: vec![] }
    }
}
impl Pattern {
    pub fn new(events: Vec<Event>) -> Self {
        Self { events }
    }

    pub fn down_mut(&mut self, duration: Duration) -> &mut Self {
        self.events.push(Event::new().down(duration));
        self
    }

    pub fn up_mut(&mut self, duration: Duration) -> &mut Self {
        self.events.push(Event::new().up(duration));
        self
    }

    pub fn void_mut(&mut self, duration: Duration) -> &mut Self {
        self.events.push(Event::new().void(duration));
        self
    }

    pub fn click_mut(&mut self) -> &mut Self {
        self.up_mut(Duration::ZERO)
            .down_mut(Duration::ZERO)
            .up_mut(Duration::ZERO)
    }

    pub fn unclick_mut(&mut self) -> &mut Self {
        self.down_mut(Duration::ZERO)
            .up_mut(Duration::ZERO)
            .down_mut(Duration::ZERO)
    }

    pub fn press_mut(&mut self) -> &mut Self {
        self.up_mut(Duration::ZERO).down_mut(Duration::ZERO)
    }

    pub fn release_mut(&mut self) -> &mut Self {
        self.down_mut(Duration::ZERO).up_mut(Duration::ZERO)
    }

    pub fn down(self, duration: Duration) -> Self {
        Self {
            events: self
                .events
                .into_iter()
                .chain(Some(Event::new().down(duration)))
                .collect(),
        }
    }

    pub fn up(self, duration: Duration) -> Self {
        Self {
            events: self
                .events
                .into_iter()
                .chain(Some(Event::new().up(duration)))
                .collect(),
        }
    }

    pub fn void(self, duration: Duration) -> Self {
        Self {
            events: self
                .events
                .into_iter()
                .chain(Some(Event::new().void(duration)))
                .collect(),
        }
    }

    pub fn click(self) -> Self {
        self.up(Duration::ZERO)
            .down(Duration::ZERO)
            .up(Duration::ZERO)
    }

    pub fn unclick(self) -> Self {
        self.down(Duration::ZERO)
            .up(Duration::ZERO)
            .down(Duration::ZERO)
    }

    pub fn press(self) -> Self {
        self.up(Duration::ZERO).down(Duration::ZERO)
    }

    pub fn release(self) -> Self {
        self.down(Duration::ZERO).up(Duration::ZERO)
    }
}

pub type InputSequence = HashMap<KeyCode, Pattern>;

// --+--- b
// -+-

// check if pattern_b is in pattern_a
pub fn pattern_match(pattern_a: &Pattern, pattern_b: &Pattern) -> bool {
    for (i, _) in pattern_a.events.iter().enumerate() {
        let mut iter_a = pattern_a.events[i..].iter().peekable();
        let mut iter_b = pattern_b.events.iter().peekable();

        while let (Some(a), Some(b)) = (iter_a.peek(), iter_b.peek()) {
            if b.state == State::VOID {
                iter_a.next();
                iter_b.next();
                continue;
            }
            if (a.state != b.state) {
                break;
            }

            if (a.duration < b.duration) {
                // normalize states before processing ? UP(1), UP(1) -> UP(2)
                break;
            }

            iter_a.next();
            iter_b.next();
        }

        if iter_b.peek().is_none() {
            return true;
        }
    }

    false
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
