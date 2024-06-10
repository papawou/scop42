use crate::{engine::Engine, graphics_pipeline::GraphicsPipeline, mesh::Mesh, vertex::Vertex};

struct MeshRenderer<'a, T> {
    graphics_pipeline: GraphicsPipeline<'a>,
    mesh: Mesh<Vertex>,
    push_constants: Option<T>,
}

impl<'a, T> MeshRenderer<'a, T> {
    fn begin_render(&self, engine: &Engine) {
        let vertex_buffers = [self.mesh.vertex_buffer.as_ref().unwrap().buffer];
        let offsets = [0];
        self.device
            .cmd_bind_vertex_buffers(cmd, 0, &vertex_buffers, &offsets);

        let push_constants = [MeshPushConstants {
            data: glam::Vec4::new(0.0, 0.0, -2.0, 0.0),
            render_matrix: glam::Mat4::IDENTITY,
        }];

        self.device.cmd_push_constants(
            cmd,
            pipeline.layout.clone(),
            vk::ShaderStageFlags::VERTEX,
            std::mem::size_of::<MeshPushConstants>() as u32,
            &vertex_buffers,
            //push_constants.as_slice(),
        );
        self.device
            .cmd_draw(cmd, self.mesh.vertices.len() as u32, 1, 0, 0);
    }
}
