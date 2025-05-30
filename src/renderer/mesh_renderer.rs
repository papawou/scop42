use std::io::Write;

use ash::vk;

use crate::{
    ft_vk::{Engine, PipelineLayout, Renderer},
    material::Material,
    mesh::Mesh,
    vertex::Vertex,
};

pub struct MeshRenderer<'a, TPushConstants, TMaterial>
where
    TPushConstants: crate::traits::IntoOwned,
{
    pub material: &'a Material<TMaterial>, // how render its called ?
    pub mesh: &'a Mesh<'a, Vertex>,
    pub push_constants: Option<TPushConstants>,

    pub pipeline_layout: &'a PipelineLayout<TPushConstants>, //used for bind
}

impl<'a, TPushConstants: crate::traits::IntoOwned> Renderer
    for MeshRenderer<'a, TPushConstants, crate::material::Pipeline>
{
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

        // pipeline_layout
        {
            if let Some(constants) = &self.push_constants {
                let tmp = constants.into_owned();
                let push_constants = crate::helpers::struct_to_bytes(&tmp);
                engine.device.cmd_push_constants(
                    cmd,
                    self.pipeline_layout.as_vk(),
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    push_constants,
                )
            };
            engine.device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout.as_vk(),
                0,
                &[self.material.descriptor_set],
                &[],
            );
        }

        engine.device.cmd_bind_index_buffer(
            cmd,
            self.mesh.index_buffer.as_ref().unwrap().buffer,
            0,
            vk::IndexType::UINT32,
        );

        engine.device.cmd_bind_pipeline(
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            self.material.pipeline.0,
        );
        engine
            .device
            .cmd_draw_indexed(cmd, self.mesh.asset.indices.len() as u32, 1, 0, 0, 0);
        engine.device.cmd_end_render_pass(cmd);
    }
}
