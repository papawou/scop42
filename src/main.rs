#![allow(warnings)]

mod ObjAsset;
mod conf;
mod engine;
mod graphics_pipeline;
mod helpers;
mod mesh;
mod mesh_constants;
mod mesh_renderer;
mod pipeline_layout;
mod traits;
mod tri_renderer;
mod vertex;

use std::path;

use anyhow::Ok;
use ash::vk::{self};
use engine::Engine;
use graphics_pipeline::{create_mesh_pipeline, create_tri_pipeline, GraphicsPipeline};
use mesh_constants::MeshConstants;
use mesh_renderer::MeshRenderer;
use pipeline_layout::{create_default_layout, create_mesh_layout};
use tri_renderer::TriRenderer;
use vertex::Vertex;
use winit::{
    event_loop::EventLoop, platform::windows::WindowExtWindows, raw_window_handle::HasWindowHandle,
};

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");


    let obj = ObjAsset::ObjAsset::parse("f 11/11// 1 4 13");

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

    let mut engine = engine::Engine::new(entry, &window);

    //MESH RENDERER
    let mut mesh = mesh::load_default_mesh(
        &engine.device,
        engine.allocator.as_mut().unwrap(),
        engine.graphics_queue,
        engine.frames[0].command_buffer,
        engine.frames[0].command_pool,
    );
    let layout = create_mesh_layout::<MeshConstants>(&engine.device);
    let mut renderer = {
        let device_address = mesh
            .vertex_buffer
            .as_ref()
            .unwrap()
            .device_address
            .as_ref()
            .unwrap();

        MeshRenderer {
            graphics_pipeline: create_mesh_pipeline(
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

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop
        .run(
            |event: winit::event::Event<_>, elwt: &winit::event_loop::EventLoopWindowTarget<_>| {
                match event {
                    winit::event::Event::AboutToWait => window.request_redraw(),
                    winit::event::Event::LoopExiting => {
                        unsafe { engine.device.device_wait_idle() }.unwrap();
                    }
                    winit::event::Event::WindowEvent { event, .. } => match event {
                        winit::event::WindowEvent::RedrawRequested => {
                            match window.is_minimized() {
                                Some(false) => (),
                                _ => return,
                            }

                            if require_resize {
                                let new_size = window.inner_size();

                                unsafe { engine.device.device_wait_idle().unwrap() }; //FLOW CONTROL wait for device no more work

                                unsafe {
                                    engine
                                        .device
                                        .destroy_pipeline(renderer.graphics_pipeline.pipeline, None)
                                };

                                unsafe { engine.handle_resize((new_size.width, new_size.height)) };

                                renderer.graphics_pipeline =
                                    graphics_pipeline::create_mesh_pipeline(
                                        &engine.device,
                                        engine.render_pass,
                                        engine.swapchain.extent,
                                        &layout,
                                    );
                            }

                            // engine loop
                            if let Some(constants) = &renderer.push_constants {
                                let updated_constants = update_mesh_constants(&engine, constants);
                                renderer.push_constants = Some(updated_constants);
                            }

                            require_resize = unsafe { engine.draw_frame(&renderer) };
                        }
                        winit::event::WindowEvent::Resized(_) => require_resize = false,
                        winit::event::WindowEvent::CloseRequested => elwt.exit(),
                        winit::event::WindowEvent::KeyboardInput {
                            event:
                                winit::event::KeyEvent {
                                    physical_key:
                                        winit::keyboard::PhysicalKey::Code(
                                            winit::keyboard::KeyCode::KeyW,
                                        ),
                                    state: winit::event::ElementState::Pressed,
                                    repeat: false,
                                    ..
                                },
                            ..
                        } => {}
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
            .destroy_pipeline(renderer.graphics_pipeline.pipeline, None)
    };
    if let Some(allocator) = &engine.allocator {
        mesh.destroy_buffers(&allocator);
    }
    unsafe { engine.device.destroy_pipeline_layout(layout.as_vk(), None) };
    unsafe { engine.destroy() };

    Ok(())
}

struct AllocatedBuffer {
    buffer: vk::Buffer,
    device_address: Option<vk::DeviceAddress>,
    buffer_size: usize,
    allocation: vk_mem::Allocation,
}

fn update_mesh_constants<'a>(engine: &Engine, constants: &MeshConstants<'a>) -> MeshConstants<'a> {
    let elapsed = engine.start_instant.elapsed().as_secs_f32();

    let mesh_matrix = {
        let cam_pos = glam::Vec3::new(0.0, 0.0, -2.0);
        let view = glam::Mat4::from_translation(cam_pos);
        let projection =
            glam::Mat4::perspective_rh_gl(70.0_f32.to_radians(), 1700.0 / 900.0, 0.1, 200.0);
        let model = glam::Mat4::from_rotation_y(elapsed * 20.0f32.to_radians());
        projection * view * model
    };

    MeshConstants {
        render_matrix: mesh_matrix,
        vertex_buffer: constants.vertex_buffer,
    }
}
