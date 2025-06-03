#![allow(warnings)]

mod camera;
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

use std::{
    path::{self, Path},
    time::{Duration, Instant},
};

use anyhow::Ok;
use ash::vk::{self};
use camera::Camera;
use ecs::{
    component::Component,
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
use winit::{event_loop::EventLoop, keyboard::KeyCode};

use crate::components::Position;
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

    let mut world = World::new();

    let camera = {
        let entity = world.spawn();
        world
            .components
            .add_component(&entity, components::Position(Vec3::ZERO));

        world.add_system(system(a_system));
        world.add_system_mut(system_mut(a_mut_system));

        entity
    };

    let mut recorder = InputRecorder::new();

    {
        // closure data
        let mut material = Some(material);

        // loop logic
        let mut require_resize = true;
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
                            winit::event::DeviceEvent::MouseMotion { delta } => {
                                //dbg!("{:?}", delta);
                            }
                            _ => {}
                        },

                        winit::event::Event::WindowEvent { event, .. } => match event {
                            // WINDOW
                            winit::event::WindowEvent::RedrawRequested => {
                                if require_resize {
                                    let new_size = window.inner_size();

                                    unsafe { engine.device.device_wait_idle().unwrap() }; // FLOW CONTROL wait for device no more work
                                    unsafe {
                                        engine.handle_resize((new_size.width, new_size.height))
                                    };
                                    material = Some(
                                        material
                                            .take()
                                            .unwrap()
                                            .unload_pipeline(&engine.device)
                                            .load_pipeline(
                                                &engine.device,
                                                engine.render_pass,
                                                engine.swapchain.extent,
                                                &pipeline_layout,
                                            ),
                                    );

                                    require_resize = false;
                                }

                                let renderer = MeshRenderer {
                                    material: material.as_ref().unwrap(),
                                    mesh: &mesh,
                                    pipeline_layout: &pipeline_layout,
                                    push_constants: {
                                        Some(MeshConstants {
                                            render_matrix: update_camera(
                                                engine.swapchain.aspect_ratio(),
                                                world
                                                    .components
                                                    .get_component::<Position>(&camera)
                                                    .unwrap()
                                                    .0,
                                            ),
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

                                require_resize = unsafe { engine.draw_frame(&renderer) };
                                window.request_redraw();
                            }
                            winit::event::WindowEvent::Resized(_) => require_resize = true,
                            winit::event::WindowEvent::CloseRequested => elwt.exit(),

                            // KEYBOARD EVENTSs
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
                                world.run_systems();
                            }
                            _ => {}
                        },
                        _ => {}
                    };
                },
            )
            .unwrap();

        material
            .unwrap()
            .unload_pipeline(&engine.device)
            .destroy(engine.allocator.as_ref().unwrap());
    }

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

    Ok(())
}

fn update_camera(aspect_ra tio: f32, pos: glam::Vec3) -> Mat4 {
    let target = glam::Vec3::new(0.0, 0.0, 0.0);
    let up = glam::Vec3::new(0.0, 1.0, 0.0);
    let view = glam::Mat4::look_at_rh(pos, target, up);
    let projection = glam::Mat4::perspective_rh(70.0_f32.to_radians(), aspect_ratio, 0.1, 200.0);

    let fix_upside = glam::Mat4 {
        y_axis: glam::vec4(0.0, -1.0, 0.0, 0.0),
        ..glam::Mat4::IDENTITY
    };
    projection * fix_upside * view
}

fn a_system(components: &ComponentsStorage, resources: &ResourceStorage) {}

fn a_mut_system(components: &mut ComponentsStorage, resources: &mut ResourceStorage) {}
