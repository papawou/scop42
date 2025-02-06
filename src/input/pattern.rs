use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use winit::keyboard::KeyCode;

pub enum InputState {
    UP,
    DOWN,
    VOID,
}

pub struct InputEvent {
    state: InputState,
    duration: Duration,
}
impl InputEvent {
    pub fn new() -> Self {
        InputEvent {
            state: InputState::DOWN,
            duration: Duration::ZERO,
        }
    }

    pub fn duration(self, duration: Duration) -> Self {
        Self { duration, ..self }
    }

    pub fn down(self, duration: Duration) -> Self {
        Self {
            state: InputState::DOWN,
            duration,
        }
    }

    pub fn up(self, duration: Duration) -> Self {
        Self {
            state: InputState::UP,
            duration,
        }
    }

    pub fn void(self, duration: Duration) -> Self {
        Self {
            state: InputState::VOID,
            duration,
        }
    }

    pub fn state(self, state: InputState) -> Self {
        Self { state, ..self }
    }

    pub fn duration_mut(&mut self, duration: Duration) -> &mut Self {
        self.duration = duration;
        self
    }

    pub fn down_mut(&mut self, duration: Duration) -> &mut Self {
        self.state = InputState::DOWN;
        self.duration = duration;
        self
    }

    pub fn up_mut(&mut self, duration: Duration) -> &mut Self {
        self.state = InputState::UP;
        self.duration = duration;
        self
    }

    pub fn void_mut(&mut self, duration: Duration) -> &mut Self {
        self.state = InputState::VOID;
        self.duration = duration;
        self
    }

    pub fn state_mut(&mut self, state: InputState) -> &mut Self {
        self.state = state;
        self
    }
}

pub struct InputPattern {
    events: Vec<InputEvent>,
}
impl Default for InputPattern {
    fn default() -> Self {
        Self { events: vec![] }
    }
}
impl InputPattern {
    pub fn new(events: Vec<InputEvent>) -> Self {
        Self { events }
    }

    pub fn down_mut(&mut self, duration: Duration) -> &mut Self {
        self.events.push(InputEvent::new().down(duration));
        self
    }

    pub fn up_mut(&mut self, duration: Duration) -> &mut Self {
        self.events.push(InputEvent::new().up(duration));
        self
    }

    pub fn void_mut(&mut self, duration: Duration) -> &mut Self {
        self.events.push(InputEvent::new().void(duration));
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

    pub fn down(self, duration: Duration) -> Self {
        Self {
            events: self
                .events
                .into_iter()
                .chain(Some(InputEvent::new().down(duration)))
                .collect(),
        }
    }

    pub fn up(self, duration: Duration) -> Self {
        Self {
            events: self
                .events
                .into_iter()
                .chain(Some(InputEvent::new().up(duration)))
                .collect(),
        }
    }

    pub fn void(self, duration: Duration) -> Self {
        Self {
            events: self
                .events
                .into_iter()
                .chain(Some(InputEvent::new().void(duration)))
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
}

pub type InputSequence = HashMap<KeyCode, InputPattern>;

//  Recorder
pub struct InputInstant {
    state: InputState,
    instant: Instant,
}

pub struct InputRecorder(HashMap<KeyCode, Vec<InputInstant>>);
impl InputRecorder {
    pub fn new() -> Self {
        Self { 0: HashMap::new() }
    }

    pub fn press(&mut self, code: KeyCode, instant: Instant) {
        self.0
            .entry(code)
            .and_modify(|stack| match stack.last() {
                Some(InputInstant {
                    state: InputState::UP,
                    ..
                }) => {
                    stack.push(InputInstant {
                        instant,
                        state: InputState::DOWN,
                    });
                }
                _ => unreachable!(),
            })
            .or_insert(vec![InputInstant {
                state: InputState::DOWN,
                instant,
            }]);
    }

    pub fn release(&mut self, code: KeyCode, instant: Instant) {
        self.0
            .entry(code)
            .and_modify(|stack| match stack.last() {
                Some(InputInstant {
                    state: InputState::DOWN,
                    ..
                }) => {
                    stack.push(InputInstant {
                        state: InputState::UP,
                        instant,
                    });
                }
                _ => unreachable!(),
            })
            .or_insert(vec![InputInstant {
                state: InputState::UP,
                instant,
            }]);
    }
}
