use ash::vk;

use crate::{
    ft_vk::{Engine, Renderer},
    material::Material,
};

pub struct TriRenderer<'a> {
    pub material: Material<'a, ()>,
}

impl<'a> Renderer for TriRenderer<'a> {
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
        engine.device.cmd_bind_pipeline(
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            self.material.pipeline,
        );
        engine.device.cmd_draw(cmd, 3, 1, 0, 0);

        engine.device.cmd_end_render_pass(cmd);
    }
}
