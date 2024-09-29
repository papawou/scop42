#![allow(warnings)]

mod conf;
mod ft_vk;
mod helpers;
pub mod material;
mod mesh;
mod mesh_constants;
pub mod obj_asset;
mod renderers;
mod traits;
mod vertex;

use std::{
    path::{self, Path},
    time::Duration,
};

use anyhow::Ok;
use ash::vk::{self};
use ft_vk::Engine;
use material::Material;
use mesh::from_obj;
use mesh_constants::MeshConstants;
use obj_asset::{ObjAssetBuilder, ObjRaw};
use renderers::{MeshRenderer, TriRenderer};
use vertex::Vertex;
use winit::event_loop::EventLoop;

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

    //MESH RENDERER
    // let mut test = mesh::load_default_mesh(
    //     &engine.device,
    //     engine.allocator.as_mut().unwrap(),
    //     engine.graphics_queue,
    //     engine.frames[0].command_buffer,
    //     engine.frames[0].command_pool,
    // );

    let mut mesh = {
        let obj_path = Path::new("resources/teapot2.obj");
        let obj = ObjRaw::load_from_file(&obj_path);
        let material_lib = obj_asset::load_materials(&obj);
        let obj_asset = ObjAssetBuilder::new(&obj).normals_from_face(true).build();
        let mut mesh = from_obj(&obj_asset);
        mesh.load(
            &engine.device,
            engine.allocator.as_mut().unwrap(),
            engine.graphics_queue,
            engine.frames[0].command_buffer,
            engine.frames[0].command_pool,
        );
        mesh
    };

    let layout = material::mesh::create_layout::<MeshConstants>(&engine.device);
    let mut renderer = {
        let device_address = mesh
            .vertex_buffer
            .as_ref()
            .unwrap()
            .device_address
            .as_ref()
            .unwrap();

        MeshRenderer {
            material: material::mesh::create_material(
                &engine.device,
                engine.render_pass,
                engine.swapchain.extent,
                &layout,
            ),
            mesh: &mesh,
            push_constants: Some(MeshConstants {
                render_matrix: glam::Mat4::IDENTITY,
                vertex_buffer: device_address,
            }),
        }
    };
    let mut require_resize = false;

    let mut camera_pos = glam::Vec3 {
        z: 2.0f32,
        ..glam::Vec3::ZERO
    };

    let mut last_update = std::time::Instant::now();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop
        .run(
            |event: winit::event::Event<_>, elwt: &winit::event_loop::EventLoopWindowTarget<_>| {
                match event {
                    winit::event::Event::LoopExiting => {
                        unsafe { engine.device.device_wait_idle() }.unwrap();
                    }
                    winit::event::Event::WindowEvent { event, .. } => match event {
                        // WINDOW
                        winit::event::WindowEvent::RedrawRequested => {
                            match window.is_minimized() {
                                Some(false) => (),
                                _ => return,
                            }

                            if require_resize {
                                let new_size = window.inner_size();

                                unsafe { engine.device.device_wait_idle().unwrap() }; // FLOW CONTROL wait for device no more work

                                unsafe {
                                    engine
                                        .device
                                        .destroy_pipeline(renderer.material.pipeline, None)
                                };

                                unsafe { engine.handle_resize((new_size.width, new_size.height)) };

                                renderer.material = material::mesh::create_material(
                                    &engine.device,
                                    engine.render_pass,
                                    engine.swapchain.extent,
                                    &layout,
                                );
                            }

                            // engine loop
                            if let Some(constants) = &renderer.push_constants {
                                let updated_constants =
                                    update_mesh_constants(&engine, camera_pos, constants);
                                renderer.push_constants = Some(updated_constants);
                            }

                            require_resize = unsafe { engine.draw_frame(&renderer) };
                            window.request_redraw();
                        }
                        winit::event::WindowEvent::Resized(_) => require_resize = true,
                        winit::event::WindowEvent::CloseRequested => elwt.exit(),

                        // CONTROLS
                        winit::event::WindowEvent::KeyboardInput {
                            event:
                                winit::event::KeyEvent {
                                    physical_key,
                                    state: winit::event::ElementState::Pressed,
                                    ..
                                },
                            ..
                        } => {
                            last_update = std::time::Instant::now();
                            let time_elapsed = last_update
                                .duration_since(engine.start_instant)
                                .min(Duration::from_millis(30));

                            match physical_key {
                                winit::keyboard::PhysicalKey::Code(
                                    winit::keyboard::KeyCode::KeyW,
                                ) => {
                                    camera_pos.z += -1.0f32 * time_elapsed.as_secs_f32();
                                }
                                winit::keyboard::PhysicalKey::Code(
                                    winit::keyboard::KeyCode::KeyS,
                                ) => {
                                    camera_pos.z += 1.0f32 * time_elapsed.as_secs_f32();
                                }
                                winit::keyboard::PhysicalKey::Code(
                                    winit::keyboard::KeyCode::KeyA,
                                ) => {
                                    camera_pos.x += -1.0f32 * time_elapsed.as_secs_f32();
                                }
                                winit::keyboard::PhysicalKey::Code(
                                    winit::keyboard::KeyCode::KeyD,
                                ) => {
                                    camera_pos.x += 1.0f32 * time_elapsed.as_secs_f32();
                                }
                                winit::keyboard::PhysicalKey::Code(
                                    winit::keyboard::KeyCode::Space,
                                ) => {
                                    camera_pos.y += 1.0f32 * time_elapsed.as_secs_f32();
                                }
                                winit::keyboard::PhysicalKey::Code(
                                    winit::keyboard::KeyCode::ControlLeft,
                                ) => {
                                    camera_pos.y += -1.0f32 * time_elapsed.as_secs_f32();
                                }
                                _ => {}
                            }
                            window.request_redraw();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            },
        )
        .unwrap();

    unsafe {
        engine
            .device
            .destroy_pipeline(renderer.material.pipeline, None)
    };
    if let Some(allocator) = &engine.allocator {
        mesh.unload(&allocator);
    }
    unsafe { engine.device.destroy_pipeline_layout(layout.as_vk(), None) };
    unsafe { engine.destroy() };

    Ok(())
}

fn update_mesh_constants<'a>(
    engine: &Engine,
    camera_pos: glam::Vec3,
    constants: &MeshConstants<'a>,
) -> MeshConstants<'a> {
    let elapsed = engine.start_instant.elapsed().as_secs_f32();

    let mesh_matrix = {
        let cam_pos = camera_pos;
        let cam_target = glam::Vec3::new(0.0, 0.0, 0.0);
        let cam_up = glam::Vec3::new(0.0, 1.0, 0.0);

        let view = glam::Mat4::look_at_rh(cam_pos, cam_target, cam_up);
        let projection = glam::Mat4::perspective_rh(
            70.0_f32.to_radians(),
            engine.swapchain.extent.width as f32 / engine.swapchain.extent.height as f32,
            0.1,
            200.0,
        );

        let fix_upside = glam::Mat4 {
            y_axis: glam::vec4(0.0, -1.0, 0.0, 0.0),
            ..glam::Mat4::IDENTITY
        };
        projection * fix_upside * view
    };

    MeshConstants {
        render_matrix: mesh_matrix,
        vertex_buffer: constants.vertex_buffer,
    }
}
