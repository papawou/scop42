mod conf;
mod engine;
mod graphics_pipeline;
mod mesh;
mod mesh_renderer;
mod pipeline_layout;
mod tri_renderer;
mod vertex;

use anyhow::Ok;
use ash::vk::{self};
use conf::MAX_FRAMES_IN_FLIGHT;
use engine::Engine;
use graphics_pipeline::{create_mesh_pipeline, create_tri_pipeline, GraphicsPipeline};
use mesh_renderer::MeshRenderer;
use pipeline_layout::{create_default_layout, create_mesh_layout, MeshPushConstants};
use tri_renderer::TriRenderer;
use vertex::Vertex;
use winit::{
    event_loop::EventLoop, platform::windows::WindowExtWindows, raw_window_handle::HasWindowHandle,
};

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
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

    //INIT RENDERER
    let mut mesh = mesh::load_default_mesh(engine.allocator.as_ref().unwrap());
    let layout = create_mesh_layout::<MeshPushConstants>(&engine.device);
    let mut renderer = MeshRenderer {
        graphics_pipeline: create_mesh_pipeline::<Vertex>(
            &engine.device,
            engine.render_pass,
            engine.swapchain.extent,
            &layout,
        ),
        mesh: &mesh,
        push_constants: Some(MeshPushConstants {
            data: glam::Vec4::new(0.0, 0.0, -2.0, 0.0),
            render_matrix: glam::Mat4::IDENTITY,
        }),
    };

    // let layout = create_default_layout(&engine.device);
    // let mut renderer = TriRenderer {
    //     graphics_pipeline: create_tri_pipeline(
    //         &engine.device,
    //         engine.render_pass,
    //         engine.swapchain.extent,
    //         &layout,
    //     ),
    // };

    let mut current_frame = 0;
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
                                    graphics_pipeline::create_mesh_pipeline::<Vertex>(
                                        &engine.device,
                                        engine.render_pass,
                                        engine.swapchain.extent,
                                        &layout,
                                    );
                            }

                            require_resize = unsafe { engine.draw_frame(current_frame, &renderer) };
                            current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
                        }
                        winit::event::WindowEvent::Resized(_) => require_resize = true,
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
        mesh.unload(&allocator);
    }
    unsafe { engine.device.destroy_pipeline_layout(layout, None) };
    unsafe { engine.destroy() };

    Ok(())
}

enum GraphicsPipelineType<'a> {
    Tri(&'a GraphicsPipeline<'a>),
    Mesh(&'a GraphicsPipeline<'a>),
    None,
}
struct GraphicsPipelineAtlas<'a> {
    tri_pipeline: GraphicsPipelineType<'a>,
    mesh_pipeline: GraphicsPipelineType<'a>,
}

struct AllocatedBuffer {
    buffer: vk::Buffer,
    allocation: vk_mem::Allocation,
}
