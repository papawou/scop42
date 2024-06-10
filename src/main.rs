mod conf;
mod engine;
mod graphics_pipeline;
mod mesh;
mod pipeline_layout;
mod utils;
mod vertex;
mod mesh_renderer;

use anyhow::Ok;
use ash::vk::{self};
use conf::MAX_FRAMES_IN_FLIGHT;
use graphics_pipeline::{create_mesh_pipeline, GraphicsPipeline};
use mesh::Mesh;
use pipeline_layout::create_mesh_layout;
use vertex::Vertex;
use winit::{platform::windows::WindowExtWindows, raw_window_handle::HasWindowHandle};

fn main() -> anyhow::Result<()> {
    //std::env::set_var("RUST_BACKTRACE", "1");
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
    let mesh = mesh::load_default_mesh(engine.allocator.as_ref().expect("No allocator"));
    let mesh_layout = create_mesh_layout(&engine.device);
    let graphics_pipeline = create_mesh_pipeline::<Vertex>(
        &engine.device,
        engine.render_pass,
        engine.swapchain.extent,
        &mesh_layout,
    );

    let mut current_frame = 0;
    let mut framebuffer_resized = false;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop
        .run(move |event, elwt| match event {
            winit::event::Event::AboutToWait => window.request_redraw(),
            winit::event::Event::LoopExiting => {
                unsafe { engine.device.device_wait_idle() }.unwrap();
                unsafe { engine.destroy() };
            }
            //WINDOW EVENTS
            winit::event::Event::WindowEvent { event, .. } => match event {
                //WINDOW MANAGENMENT
                winit::event::WindowEvent::RedrawRequested => {
                    if framebuffer_resized && unsafe { engine.handle_resize(&window) } {
                        return;
                    }
                    framebuffer_resized = unsafe { engine.draw_frame(current_frame) };
                    current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
                }
                winit::event::WindowEvent::Resized(_) => framebuffer_resized = true,
                winit::event::WindowEvent::CloseRequested => elwt.exit(),
                //CONTROLS
                winit::event::WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key:
                                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyW),
                            state: winit::event::ElementState::Pressed,
                            repeat: false,
                            ..
                        },
                    ..
                } => {}
                _ => {}
            },
            _ => {}
        })
        .unwrap();

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
