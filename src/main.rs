#![allow(warnings)]

mod components;
mod conf;
mod ft_vk;
mod helpers;
mod input;
pub mod material;
mod material_asset;
mod mesh;
mod mesh_asset;
mod mesh_constants;
pub mod obj_asset;
mod physics;
mod renderer;
mod traits;
mod vertex;
mod window;

use std::{
    path::{self, Path},
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::Ok;
use ash::vk::{self};
use ecs::{
    component::Component,
    entity::Entity,
    macros::Component,
    resource::ResourceStorage,
    storage::ComponentsStorage,
    system::{system, system_mut},
    world::{self, World},
};
use ft_vk::{
    descriptor_allocator::DescriptorAllocator,
    descriptor_set_layout::{self, DescriptorSetLayoutCreateInfoBuilder},
    PipelineLayout,
};
use glam::{Mat4, Quat, Vec3, Vec3Swizzles};
use input::recorder::InputRecorder;
use material::Material;
use material_asset::MaterialAsset;
use mesh::Mesh;
use mesh_asset::MeshAsset;
use mesh_constants::MeshConstants;
use obj_asset::{ObjAssetBuilder, ObjRaw};
use renderer::MeshRenderer;
use vertex::Vertex;
use winit::{dpi::PhysicalSize, event_loop::EventLoop, keyboard::KeyCode};

use crate::{
    components::{Camera, Direction, PhysicsBody, Position},
    input::{input::InputEnum, recorder, recorder_to_queue},
    material::Pipeline,
    physics::{compute_position, compute_velocity, traits::IntegrateFn},
};

//platform::wayland::WindowBuilderExtWayland

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let entry = unsafe { ash::Entry::load()? };

    //window
    let event_loop = winit::event_loop::EventLoop::new()?;
    let window = winit::window::WindowBuilder::new()
        .with_title("Hello window!")
        .with_inner_size(winit::dpi::LogicalSize::new(
            conf::WINDOW_WIDTH,
            conf::WINDOW_HEIGHT,
        ))
        .build(&event_loop)?;

    let mut render_engine = ft_vk::Engine::new(entry, &window);
    let mut physics_engine = physics::Engine {
        frame_time_acc: Duration::ZERO,
        last_update: Instant::now(),
    };

    // assets
    let obj = {
        let obj_path = Path::new("resources/teapot2.obj");
        ObjRaw::load_from_file(&obj_path).optimise_positions()
    };
    let obj_asset = ObjAssetBuilder::new(&obj).build();
    let material_libs = obj_asset::load_materials(&obj);

    // mesh
    let mesh_asset = MeshAsset::from_obj(&obj_asset);
    let mut mesh = {
        let mut mesh = Mesh {
            asset: &mesh_asset,
            index_buffer: None,
            vertex_buffer: None,
        };
        mesh.load(
            &render_engine.device,
            render_engine.allocator.as_mut().unwrap(),
            render_engine.graphics_queue,
            render_engine.frames[0].command_buffer,
            render_engine.frames[0].command_pool,
        );
        mesh
    };
    let material_asset: MaterialAsset = material_libs
        .values()
        .flat_map(|mat_lib| mat_lib.materials.values())
        .next()
        .unwrap() //thrown when no material is defined in obj file, should be fallback to a default material (ambient 1.0)
        .clone()
        .into();
    let material_set_layout = material::descriptor_set_layout(&render_engine.device);
    // pipeline_layout
    let push_constant_ranges = [
        // scene constants (render_matrix / mesh_buffer_address)
        vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .size(std::mem::size_of::<MeshConstants>() as u32),
    ];
    let pipeline_layout = PipelineLayout::<MeshConstants> {
        layout: {
            let set_layouts = [material_set_layout];
            let info = vk::PipelineLayoutCreateInfo::default()
                .push_constant_ranges(&push_constant_ranges)
                .set_layouts(&set_layouts);
            unsafe {
                render_engine
                    .device
                    .create_pipeline_layout(&info, None)
                    .unwrap()
            }
        },
        _marker: std::marker::PhantomData,
    };
    // material
    let material = Material::new(&mut render_engine, &material_asset, material_set_layout)
        .load_pipeline(
            &render_engine.device,
            render_engine.render_pass,
            render_engine.swapchain.extent,
            &pipeline_layout,
        );

    let mut world = {
        let mut world = World::new();

        //Systems
        {}
        // Origin entity
        {
            world.spawn(Some(Entity::Origin));
            world
                .components
                .add_component::<Position>(&Entity::Origin, Position(Vec3::ZERO));
        }
        // Resources
        {
            world.resources.add(InputRecorder::new());
        }
        // Camera entity
        {
            world.spawn(Some(ecs::Entity::Camera)).unwrap();
            world.components.add_component(
                &Entity::Camera,
                components::Position(Vec3::ZERO.with_z(5.0f32)),
            );
            world.components.add_component(
                &Entity::Camera,
                components::Camera {
                    aspect_ratio: render_engine.swapchain.aspect_ratio(),
                    look_at: Some(Entity::Origin),
                    fov: 90.0f32,
                    near: 0.1f32,
                    far: 200.0f32,
                },
            );
            world
                .components
                .add_component(&Entity::Camera, components::Direction(Vec3::NEG_Z));
            world.components.add_component(
                &Entity::Camera,
                components::PhysicsBody {
                    acceleration: Vec3::ZERO,
                    velocity: Vec3::ZERO,
                    integrate: Some(Box::new(|entity, world| {
                        impl<F> IntegrateFn for F
                        where
                            F: FnMut(Duration),
                        {
                            fn integrate(&mut self, dt: Duration) {
                                self(dt)
                            }
                        };

                        Box::new(|dt: Duration| {
                            let cam = unsafe {
                                world
                                    .as_unsafe_mut()
                                    .components
                                    .get_component::<Camera>(entity)
                                    .unwrap()
                            };
                            let position = unsafe {
                                world
                                    .as_unsafe_mut()
                                    .components
                                    .get_component_mut::<Position>(entity)
                                    .unwrap()
                            };
                            let body = unsafe {
                                world
                                    .as_unsafe_mut()
                                    .components
                                    .get_component_mut::<PhysicsBody>(entity)
                                    .unwrap()
                            };

                            let direction = match &cam.look_at {
                                Some(target_entity) => {
                                    let target_position = unsafe {
                                        world
                                            .as_unsafe_mut()
                                            .components
                                            .get_component::<Position>(target_entity)
                                            .unwrap()
                                    };
                                    (target_position.0 - position.0).normalize()
                                }
                                None => {
                                    let direction = unsafe {
                                        world
                                            .as_unsafe_mut()
                                            .components
                                            .get_component::<Direction>(entity)
                                            .unwrap_or(&Direction(Vec3::NEG_Z))
                                    };
                                    direction.0
                                }
                            };

                            let rot = Quat::from_rotation_arc(Vec3::NEG_Z, direction);

                            position.0 += rot * (body.velocity * dt.as_secs_f32());
                        })
                    })),
                },
            );

            world.components.add_component(
                &Entity::Camera,
                components::Input(Rc::new(|entity, world| {
                    let input_recorder = world
                        .resources
                        .get::<InputRecorder>()
                        .ok()
                        .flatten()
                        .unwrap();
                    let physics_body = world
                        .components
                        .get_component_mut::<PhysicsBody>(&entity)
                        .unwrap();

                    let mut velocity = Vec3::ZERO;
                    match input_recorder.last(&KeyCode::KeyW) {
                        Some(InputEnum::Down(_)) => {
                            velocity += Vec3::NEG_Z;
                        }
                        _ => {}
                    }
                    match input_recorder.last(&KeyCode::KeyS) {
                        Some(InputEnum::Down(_)) => {
                            velocity += Vec3::Z;
                        }
                        _ => {}
                    }
                    match input_recorder.last(&KeyCode::KeyA) {
                        Some(InputEnum::Down(_)) => {
                            velocity += Vec3::NEG_X;
                        }
                        _ => {}
                    }
                    match input_recorder.last(&KeyCode::KeyD) {
                        Some(InputEnum::Down(_)) => {
                            velocity += Vec3::X;
                        }
                        _ => {}
                    }
                    match input_recorder.last(&KeyCode::Space) {
                        Some(InputEnum::Down(_)) => {
                            velocity += Vec3::Y;
                        }
                        _ => {}
                    }
                    match input_recorder.last(&KeyCode::ControlLeft) {
                        Some(InputEnum::Down(_)) => {
                            velocity += Vec3::NEG_Y;
                        }
                        _ => {}
                    }

                    physics_body.velocity = velocity;
                })),
            );
        }
        world
    };

    {
        // closure data
        let mut material = Some(material);

        // loop logic
        let mut require_resize: Option<window::Size> = None;
        let mut last_update = std::time::Instant::now();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop
            .run(
                |event: winit::event::Event<_>,
                 elwt: &winit::event_loop::EventLoopWindowTarget<_>| {
                    match event {
                        winit::event::Event::LoopExiting => {
                            unsafe { render_engine.device.device_wait_idle() }.unwrap();
                        }
                        // DEVICE
                        winit::event::Event::DeviceEvent { event, .. } => match event {
                            winit::event::DeviceEvent::MouseMotion { delta } => {}
                            _ => {}
                        },
                        winit::event::Event::WindowEvent { event, .. } => match event {
                            // WINDOW
                            winit::event::WindowEvent::RedrawRequested => {
                                let render_matrix = {
                                    let position = world
                                        .components
                                        .get_component::<Position>(&Entity::Camera)
                                        .unwrap();
                                    let direction = world
                                        .components
                                        .get_component::<Direction>(&Entity::Camera)
                                        .unwrap_or(&Direction(Vec3::NEG_Z));
                                    let camera = world
                                        .components
                                        .get_component::<Camera>(&Entity::Camera)
                                        .unwrap();

                                    let view = {
                                        match &camera.look_at {
                                            Some(target_entity) => {
                                                let target_position = world
                                                    .components
                                                    .get_component::<Position>(&target_entity)
                                                    .unwrap();

                                                glam::Mat4::look_at_rh(
                                                    position.0,
                                                    target_position.0,
                                                    glam::Vec3::Y,
                                                )
                                            }
                                            None => glam::Mat4::from_quat(Quat::from_rotation_arc(
                                                Vec3::NEG_Z,
                                                direction.0,
                                            )),
                                        }
                                    };

                                    let projection = glam::Mat4::perspective_rh(
                                        camera.fov.to_radians(),
                                        camera.aspect_ratio,
                                        camera.near,
                                        camera.far,
                                    );

                                    let fix_upside = glam::Mat4 {
                                        y_axis: glam::Vec4::NEG_Y,
                                        ..glam::Mat4::IDENTITY
                                    };

                                    projection * fix_upside * view
                                };

                                let renderer = MeshRenderer {
                                    material: material.as_ref().unwrap(),
                                    mesh: &mesh,
                                    pipeline_layout: &pipeline_layout,
                                    push_constants: {
                                        Some(MeshConstants {
                                            render_matrix,
                                            vertex_buffer: mesh
                                                .vertex_buffer
                                                .as_ref()
                                                .unwrap()
                                                .device_address
                                                .as_ref()
                                                .unwrap(),
                                        })
                                    },
                                };

                                match unsafe { render_engine.draw_frame(&renderer) } {
                                    Err(vk::Result::ERROR_OUT_OF_DATE_KHR)
                                    | Err(vk::Result::SUBOPTIMAL_KHR) => {
                                        let window_size = window.inner_size();
                                        require_resize = Some(window::Size {
                                            width: window_size.width,
                                            height: window_size.height,
                                        });
                                    }
                                    Err(e) => panic!("{:?}", e),
                                    _ => {}
                                }

                                window.request_redraw();
                            }
                            winit::event::WindowEvent::Resized(new_size) => {
                                require_resize = Some(window::Size {
                                    width: new_size.width,
                                    height: new_size.height,
                                });
                            }
                            winit::event::WindowEvent::CloseRequested => elwt.exit(),

                            // KEYBOARD EVENTS
                            winit::event::WindowEvent::KeyboardInput {
                                event:
                                    winit::event::KeyEvent {
                                        physical_key: winit::keyboard::PhysicalKey::Code(code),
                                        state,
                                        ..
                                    },
                                ..
                            } => {
                                let recorder = world
                                    .resources
                                    .get_mut::<InputRecorder>()
                                    .ok()
                                    .flatten()
                                    .unwrap();
                                match state {
                                    winit::event::ElementState::Pressed => {
                                        recorder.press(code, Instant::now());
                                    }
                                    winit::event::ElementState::Released => {
                                        recorder.release(code, Instant::now());
                                    }
                                };
                                window.request_redraw();
                            }
                            _ => {}
                        },
                        _ => {}
                    };

                    if let Some(new_size) = require_resize {
                        on_resize(
                            &mut material,
                            &pipeline_layout,
                            &mut world,
                            &mut render_engine,
                            new_size,
                        );
                        require_resize = None;
                    }

                    // Loop logic
                    process_input(&mut world);
                    physics_system(&mut world, &mut physics_engine);
                },
            )
            .unwrap();

        material
            .unwrap()
            .unload_pipeline(&render_engine.device)
            .destroy(render_engine.allocator.as_ref().unwrap());
    }

    // Clean
    {
        unsafe {
            render_engine
                .device
                .destroy_descriptor_set_layout(material_set_layout, None)
        };

        if let Some(allocator) = &render_engine.allocator {
            mesh.unload(&allocator);
        }
        unsafe {
            render_engine
                .device
                .destroy_pipeline_layout(pipeline_layout.as_vk(), None)
        };

        unsafe { render_engine.destroy() };
    }

    Ok(())
}

// Handle window resize events and update the engine, material, and camera accordingly.
fn on_resize<TPipelineLayout>(
    material: &mut Option<Material<Pipeline>>,
    pipeline_layout: &PipelineLayout<TPipelineLayout>,
    world: &mut World,
    render_engine: &mut ft_vk::Engine,
    new_size: window::Size,
) {
    // Engine
    unsafe { render_engine.device.device_wait_idle() };
    unsafe { render_engine.handle_resize((new_size.width, new_size.height)) };

    // Material
    *material = Some(
        material
            .take()
            .unwrap()
            .unload_pipeline(&render_engine.device)
            .load_pipeline(
                &render_engine.device,
                render_engine.render_pass,
                render_engine.swapchain.extent,
                pipeline_layout,
            ),
    );

    // Camera
    world
        .components
        .get_component_mut::<components::Camera>(&Entity::Camera)
        .unwrap()
        .aspect_ratio = render_engine.swapchain.aspect_ratio();
}

// Process input events and apply them to the world.
fn process_input(world: &mut World) {
    let storage = world
        .components
        .get_component_storage::<components::Input>()
        .cloned();

    if let Some(storage) = storage {
        for (entity, input) in storage {
            input.apply(&entity, world);
        }
    }
}

fn physics_system(world: &mut World, engine: &mut physics::Engine) {
    let world = world as *mut World;

    let bodies = unsafe {
        (*world)
            .components
            .get_component_storage::<PhysicsBody>()
            .unwrap()
            .iter()
            .map(|(entity, physics_body)| physics_body.integrate(entity, &mut *world))
            .collect()
    };

    engine.tick(bodies);
}
