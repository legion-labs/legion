#![allow(unsafe_code)]

use lgn_ecs::prelude::Entity;
use lgn_embedded_fs::embedded_watched_file;
use lgn_graphics_api::{
    BlendState, ColorClearValue, ColorRenderTargetBinding, CompareOp, DepthState,
    DepthStencilClearValue, DepthStencilRenderTargetBinding, Format, GraphicsPipelineDef, LoadOp,
    PrimitiveTopology, RasterizerState, ResourceState, SampleCount, StencilOp, StoreOp,
    VertexAttributeRate, VertexLayout, VertexLayoutAttribute, VertexLayoutBuffer,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_tracing::span_fn;

use crate::{
    cgen,
    components::{RenderSurface, VisualComponent},
    gpu_renderer::{GpuInstanceManager, MeshRenderer, RenderLayer},
    hl_gfx_api::HLCommandBuffer,
    resources::{MeshManager, PipelineHandle, PipelineManager},
    RenderContext,
};

pub struct TmpRenderPass {
    pub color: [f32; 4],
    pub speed: f32,
}

embedded_watched_file!(INCLUDE_BRDF, "gpu/include/brdf.hsh");
embedded_watched_file!(INCLUDE_MESH, "gpu/include/mesh.hsh");

impl TmpRenderPass {
    pub fn new(pipeline_manager: &PipelineManager) -> Self {
        Self {
            color: [0f32, 0f32, 0.2f32, 1.0f32],
            speed: 1.0f32,
        }
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    #[span_fn]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        instance_manager: &GpuInstanceManager,
        render_surface: &mut RenderSurface,
        mesh_renerer: &MeshRenderer,
    ) {
        let pipeline = render_context
            .pipeline_manager()
            .get_pipeline(self.pipeline_handle)
            .unwrap();

        render_surface.transition_to(cmd_buffer, ResourceState::RENDER_TARGET);

        cmd_buffer.begin_render_pass(
            &[ColorRenderTargetBinding {
                texture_view: render_surface.render_target_view(),
                load_op: LoadOp::Clear,
                store_op: StoreOp::Store,
                clear_value: ColorClearValue([0.2, 0.2, 0.2, 1.0]),
            }],
            &Some(DepthStencilRenderTargetBinding {
                texture_view: render_surface.depth_stencil_texture_view(),
                depth_load_op: LoadOp::Clear,
                stencil_load_op: LoadOp::DontCare,
                depth_store_op: StoreOp::Store,
                stencil_store_op: StoreOp::DontCare,
                clear_value: DepthStencilClearValue {
                    depth: 1.0,
                    stencil: 0,
                },
            }),
        );

        //cmd_buffer.bind_pipeline(pipeline);
        cmd_buffer.bind_descriptor_set_handle(render_context.frame_descriptor_set_handle());
        cmd_buffer.bind_descriptor_set_handle(render_context.view_descriptor_set_handle());

        render_set.draw(cmd_buffer, None, None);

        // for (_index, (entity, static_mesh)) in static_meshes.iter().enumerate() {
        //     if let Some(list) = instance_manager.id_va_list(*entity) {
        //         for (gpu_instance_id, _) in list {
        //             let num_vertices = mesh_manager
        //                 .mesh_from_id(static_mesh.mesh_id as u32)
        //                 .num_vertices() as u32;
        //             cmd_buffer.draw_instanced(num_vertices, 0, 1, *gpu_instance_id);
        //         }
        //     }
        // }

        cmd_buffer.end_render_pass();
    }
}
