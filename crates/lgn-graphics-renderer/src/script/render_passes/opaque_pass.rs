use lgn_graphics_api::{ColorClearValue, CommandBuffer};
use lgn_graphics_data::Color;

use crate::{
    core::{
        RenderGraphBuilder, RenderGraphContext, RenderGraphExecuteContext, RenderGraphLoadState,
        RenderGraphResourceId, RenderGraphViewId, TmpDrawContext, RENDER_LAYER_OPAQUE,
    },
    gpu_renderer::{GpuInstanceManager, MeshRenderer},
    resources::UnifiedStaticBuffer,
};

pub struct OpaqueLayerPass;

impl OpaqueLayerPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        depth_view_id: RenderGraphViewId,
        gbuffer_view_ids: [RenderGraphViewId; 4],
        draw_count_buffer_id: RenderGraphResourceId,
        draw_args_buffer_id: RenderGraphResourceId,
    ) -> RenderGraphBuilder<'a> {
        builder.add_graphics_pass("OpaqueLayer", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(
                    0,
                    gbuffer_view_ids[0],
                    RenderGraphLoadState::ClearColor(ColorClearValue(
                        Color::new(180, 180, 180, 255).as_linear().into(),
                    )),
                )
                //                .render_target(
                //                    1,
                //                    gbuffer_view_id,
                //                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
                //                )
                //                .render_target(
                //                    2,
                //                    gbuffer_view_id,
                //                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
                //                )
                //                .render_target(
                //                    3,
                //                    gbuffer_view_id,
                //                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
                //                )
                .depth_stencil(depth_view_id, RenderGraphLoadState::Load)
                .execute(move |context, execute_context, cmd_buffer| {
                    //< TMP
                    let tmp_render_list =
                        execute_context.render_list_set.get(0, RENDER_LAYER_OPAQUE);
                    let mut tmp_draw_context = TmpDrawContext {};
                    tmp_render_list.execute(&mut tmp_draw_context);
                    //> TMP

                    Self::execute_opaque_layer_pass(
                        context,
                        execute_context,
                        cmd_buffer,
                        draw_count_buffer_id,
                        draw_args_buffer_id,
                    );
                })
        })
    }

    fn execute_opaque_layer_pass(
        context: &RenderGraphContext,
        execute_context: &mut RenderGraphExecuteContext<'_, '_>,
        cmd_buffer: &mut CommandBuffer,
        draw_count_buffer_id: RenderGraphResourceId,
        draw_args_buffer_id: RenderGraphResourceId,
    ) {
        let render_context = &execute_context.render_context;
        let mesh_renderer = execute_context.render_resources.get::<MeshRenderer>();

        let static_buffer = execute_context
            .render_resources
            .get::<UnifiedStaticBuffer>();

        cmd_buffer.cmd_bind_index_buffer(static_buffer.index_buffer_binding());
        cmd_buffer.cmd_bind_vertex_buffers(
            0,
            &[execute_context
                .render_resources
                .get::<GpuInstanceManager>()
                .vertex_buffer_binding()],
        );

        mesh_renderer.render_layer_batches[RENDER_LAYER_OPAQUE.index()].draw(
            render_context,
            cmd_buffer,
            Some(context.get_buffer(draw_args_buffer_id)),
            Some(context.get_buffer(draw_count_buffer_id)),
        );
    }
}
