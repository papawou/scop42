// onResize

use ecs::{entity::EntityTag, world::World, Entity};
use glam::{Mat4, Quat};

use crate::{
    components::{self, Direction, Position},
    ft_vk::Engine,
};

pub fn on_resize(world: &mut World, engine: &Engine) {
    {
        let camera_entity = world.entities.get(&EntityTag::Camera).unwrap();
        world
            .components
            .get_component_mut::<components::Camera>(&camera_entity)
            .unwrap()
            .aspect_ratio = engine.swapchain.aspect_ratio();
    }
}

pub fn transform(world: &World, entity: &Entity) -> Mat4 {
    let position = world.components.get_component::<Position>(entity).unwrap();
    let direction = world
        .components
        .get_component::<Direction>(entity)
        .copied()
        .unwrap_or(Direction(Quat::IDENTITY));

    Mat4::from_translation(position.0) * Mat4::from_quat(direction.0)
}

pub fn camera_transform() -> Mat4 {
    let up = glam::Vec3::new(0.0, 1.0, 0.0);
    let view = glam::Mat4::look_at_rh(pos, target, up);
    let projection = glam::Mat4::perspective_rh(70.0_f32.to_radians(), aspect_ratio, 0.1, 200.0);

    let fix_upside = glam::Mat4 {
        y_axis: glam::vec4(0.0, -1.0, 0.0, 0.0),
        ..glam::Mat4::IDENTITY
    };
    projection * fix_upside * view
}
