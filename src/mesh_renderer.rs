use std::io::Write;

use ash::vk;

use crate::{
    ft_vk::{Engine, Renderer},
    material::Material,
    mesh::Mesh,
    vertex::Vertex,
};

pub struct MeshRenderer<'a, T>
where
    T: crate::traits::IntoOwned,
{
    pub material: Material<'a, T>,
    pub mesh: &'a Mesh<Vertex>,
    pub push_constants: Option<T>,
}

impl<'a, T: crate::traits::IntoOwned> Renderer for MeshRenderer<'a, T> {
    unsafe fn render(&self, engine: &Engine, framebuffer: vk::Framebuffer, cmd: vk::CommandBuffer) {
        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0f32, 0.0f32, 0.0f32, 1.0f32],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0f32,
                stencil: 0,
            },
        };

        let clear_values = [color_clear_value, depth_clear_value];

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

        if let Some(constants) = &self.push_constants {
            let tmp = constants.into_owned();
            let push_constants = crate::helpers::struct_to_bytes(&tmp);
            engine.device.cmd_push_constants(
                cmd,
                self.material.layout.as_vk(),
                vk::ShaderStageFlags::VERTEX,
                0,
                push_constants,
            )
        };

        engine.device.cmd_bind_index_buffer(
            cmd,
            self.mesh.index_buffer.as_ref().unwrap().buffer,
            0,
            vk::IndexType::UINT32,
        );

        engine.device.cmd_bind_pipeline(
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            self.material.pipeline,
        );
        engine
            .device
            .cmd_draw_indexed(cmd, self.mesh.indices.len() as u32, 1, 0, 0, 0);
        engine.device.cmd_end_render_pass(cmd);
    }
}
