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
    pub state: State,
    pub duration: Duration,
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
    pub events: Vec<Event>,
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
