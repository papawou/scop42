use std::{
    ops::{Add, Sub},
    time::Duration,
};

mod conf;

use ecs::{storage::ComponentsStorage, world::World};
use glam::Vec3;

use crate::{
    components::{position, PhysicsBody, Position},
    physics,
};

pub struct Engine {
    pub frame_time_acc: Duration,
    pub last_update: std::time::Instant,
}

impl Engine {
    pub fn run(&mut self, world: &mut World) {
        let frame_time = self.last_update.elapsed().min(conf::MAX_FRAME_TIME);
        self.last_update = std::time::Instant::now();

        self.frame_time_acc = self.frame_time_acc.add(frame_time);

        while (self.frame_time_acc >= conf::PHYSICS_FPS) {
            let components_ptr = &mut world.components as *mut ComponentsStorage;
            let physics_bodies = unsafe {
                (*components_ptr)
                    .get_component_storage_mut::<PhysicsBody>()
                    .unwrap()
            };

            for (entity, physics_body) in physics_bodies.iter_mut() {
                let position = unsafe {
                    // split borrow, because Position !== PhysicsBody
                    (*components_ptr)
                        .get_component_mut::<Position>(entity)
                        .unwrap()
                };

                physics_body.velocity +=
                    physics_body.acceleration * conf::PHYSICS_FPS.as_secs_f32();
                position.0 += physics_body.velocity * conf::PHYSICS_FPS.as_secs_f32();
            }

            self.frame_time_acc = self.frame_time_acc.sub(conf::PHYSICS_FPS);
        }

        //todo!("Alpha rendering logic here");
        // let alpha = self.frame_time_acc.div_duration_f64(conf::PHYSICS_FPS);
    }
}
