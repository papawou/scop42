use std::{
    ops::{Add, Sub},
    time::Duration,
};

use glam::Vec3;

struct Engine {
    previous_state: Vec3, // physics state
    new_state: Vec3,

    frame_time_acc: Duration,
    last_update: std::time::Instant,
}

const MAX_FRAME_TIME: Duration = Duration::from_millis(250); //ms

const PHYSICS_FPS: Duration = Duration::from_millis(16); // 60fps (1/60)

impl Engine {
    pub fn update(&mut self) {
        let frame_time = self.last_update.elapsed().min(MAX_FRAME_TIME);
        self.last_update = std::time::Instant::now();

        self.frame_time_acc = self.frame_time_acc.add(frame_time);

        while (self.frame_time_acc >= PHYSICS_FPS) {
            // do your physics brah

            self.frame_time_acc = self.frame_time_acc.sub(PHYSICS_FPS);
        }

        // PREV STATE - RENDER STATE - (FINAL) STATE
        let alpha = self.frame_time_acc.div_duration_f64(PHYSICS_FPS);

        // do your render state using alpha brah
    }
}
