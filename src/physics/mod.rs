use std::{
    ops::{Add, Sub},
    time::Duration,
};

mod conf;

use glam::Vec3;

struct Engine {
    previous_state: Vec3, // physics state
    new_state: Vec3,

    frame_time_acc: Duration,
    last_update: std::time::Instant,
}

impl Engine {
    pub fn update(&mut self) {
        let frame_time = self.last_update.elapsed().min(conf::MAX_FRAME_TIME);
        self.last_update = std::time::Instant::now();

        self.frame_time_acc = self.frame_time_acc.add(frame_time);

        while (self.frame_time_acc >= conf::PHYSICS_FPS) {
            // do your physics brah

            self.frame_time_acc = self.frame_time_acc.sub(conf::PHYSICS_FPS);
        }

        // PREV STATE - RENDER STATE - (FINAL) STATE
        let alpha = self.frame_time_acc.div_duration_f64(conf::PHYSICS_FPS);

        // do your render state using alpha brah
    }
}
