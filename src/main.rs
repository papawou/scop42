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
mod systems;
mod traits;
mod vertex;
mod window;

use std::{
    path::{self, Path},
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
    world::World,
};
use ft_vk::{
    descriptor_allocator::DescriptorAllocator,
    descriptor_set_layout::{self, DescriptorSetLayoutCreateInfoBuilder},
    Engine, PipelineLayout,
};
use glam::{Mat4, Quat, Vec3};
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
    components::{Camera, Direction, Position},
    material::Pipeline,
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

    let mut engine = ft_vk::Engine::new(entry, &window);

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
            &engine.device,
            engine.allocator.as_mut().unwrap(),
            engine.graphics_queue,
            engine.frames[0].command_buffer,
            engine.frames[0].command_pool,
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

    // descriptor_set_layout
    let material_set_layout = material::descriptor_set_layout(&engine.device);

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
            unsafe { engine.device.create_pipeline_layout(&info, None).unwrap() }
        },
        _marker: std::marker::PhantomData,
    };

    // material
    let material = Material::new(&mut engine, &material_asset, material_set_layout).load_pipeline(
        &engine.device,
        engine.render_pass,
        engine.swapchain.extent,
        &pipeline_layout,
    );

    let mut world = {
        let mut world = World::new();

        {
            //Systems
            world.add_system(system(systems::a_system));
            world.add_system_mut(system_mut(systems::a_mut_system));
        }
        {
            //Origin entity
            world.spawn(Some(Entity::Origin));
            world
                .components
                .add_component::<Position>(&Entity::Origin, Position(Vec3::ZERO));
        }
        {
            //Camera entity
            world.spawn(Some(ecs::Entity::Camera)).unwrap();
            world.components.add_component(
                &Entity::Camera,
                components::Position(Vec3::ZERO.with_z(5.0f32)),
            );
            world.components.add_component(
                &Entity::Camera,
                components::Camera {
                    aspect_ratio: engine.swapchain.aspect_ratio(),
                    look_at: Some(Entity::Origin),
                    fov: 90.0f32,
                    near: 0.1f32,
                    far: 200.0f32,
                },
            );
            world
                .components
                .add_component(&Entity::Camera, components::Direction(Quat::IDENTITY));
        }
        world
    };

    let mut recorder = InputRecorder::new();

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
                            unsafe { engine.device.device_wait_idle() }.unwrap();
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
                                        .unwrap_or(&Direction(Quat::IDENTITY));
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
                                            None => glam::Mat4::from_quat(direction.0),
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

                                match unsafe { engine.draw_frame(&renderer) } {
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
                            &mut engine,
                            new_size,
                        );
                        require_resize = None;
                    }
                },
            )
            .unwrap();

        material
            .unwrap()
            .unload_pipeline(&engine.device)
            .destroy(engine.allocator.as_ref().unwrap());
    }
    // clean
    {
        unsafe {
            engine
                .device
                .destroy_descriptor_set_layout(material_set_layout, None)
        };

        if let Some(allocator) = &engine.allocator {
            mesh.unload(&allocator);
        }
        unsafe {
            engine
                .device
                .destroy_pipeline_layout(pipeline_layout.as_vk(), None)
        };

        unsafe { engine.destroy() };
    }

    Ok(())
}

fn on_resize<TPipelineLayout>(
    material: &mut Option<Material<Pipeline>>,
    pipeline_layout: &PipelineLayout<TPipelineLayout>,
    world: &mut World,
    engine: &mut Engine,
    new_size: window::Size,
) {
    // Engine
    unsafe { engine.device.device_wait_idle() };
    unsafe { engine.handle_resize((new_size.width, new_size.height)) };

    // Material
    *material = Some(
        material
            .take()
            .unwrap()
            .unload_pipeline(&engine.device)
            .load_pipeline(
                &engine.device,
                engine.render_pass,
                engine.swapchain.extent,
                pipeline_layout,
            ),
    );

    // Camera component
    world
        .components
        .get_component_mut::<components::Camera>(&Entity::Camera)
        .unwrap()
        .aspect_ratio = engine.swapchain.aspect_ratio();
}

fn camera_move(world: &mut World) {}
