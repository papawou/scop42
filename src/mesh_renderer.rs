use ash::vk;

use crate::{
    engine::{Engine, Renderer},
    graphics_pipeline::GraphicsPipeline,
    mesh::Mesh,
    vertex::Vertex,
};

pub struct MeshRenderer<'a, T>
where
    T: Copy,
{
    pub graphics_pipeline: GraphicsPipeline<'a>,
    pub mesh: &'a Mesh<Vertex>,
    pub push_constants: Option<T>,
}

impl<'a, T: Copy> Renderer for MeshRenderer<'a, T> {
    unsafe fn render(&self, engine: &Engine, framebuffer: vk::Framebuffer, cmd: vk::CommandBuffer) {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0f32, 0.0f32, 0.0f32, 1.0f32],
            },
        }];

        let renderpass_info = vk::RenderPassBeginInfo::default()
            .render_pass(engine.render_pass)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: engine.swapchain.extent,
            })
            .framebuffer(framebuffer)
            .clear_values(&clear_values);
        engine
            .device
            .cmd_begin_render_pass(cmd, &renderpass_info, vk::SubpassContents::INLINE);

        //renderer
        let vertex_buffers = [self.mesh.vertex_buffer.as_ref().unwrap().buffer];
        let offsets = [0];
        engine
            .device
            .cmd_bind_vertex_buffers(cmd, 0, &vertex_buffers, &offsets);

        let push_constants = match self.push_constants {
            Some(push_constants) => vec![push_constants],
            _ => vec![],
        };

        let push_constants = struct_to_bytes(&push_constants);

        engine.device.cmd_push_constants(
            cmd,
            self.graphics_pipeline.layout.clone(),
            vk::ShaderStageFlags::VERTEX,
            0,
            push_constants,
        );

        engine.device.cmd_bind_pipeline(
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            self.graphics_pipeline.pipeline,
        );
        engine
            .device
            .cmd_draw(cmd, self.mesh.vertices.len() as u32, 1, 0, 0);

        engine.device.cmd_end_render_pass(cmd);
    }
}

fn struct_to_bytes<T>(s: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((s as *const T) as *const u8, std::mem::size_of::<T>()) }
}
