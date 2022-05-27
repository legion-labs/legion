use crate::{
    components::RenderSurfaceExtents,
    core::{RenderGraphBuilder, RenderGraphLoadState, RenderGraphViewId},
    resources::{MeshManager, ModelManager},
    script::RenderView,
};

pub struct DebugPass;

impl DebugPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        view: &RenderView<'_>,
        depth_view_id: RenderGraphViewId,
        radiance_write_rt_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        let view_target_extents = *view.target.extents();

        builder.add_graphics_pass("Debug", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(0, radiance_write_rt_view_id, RenderGraphLoadState::Load)
                .depth_stencil(depth_view_id, RenderGraphLoadState::Load)
                .execute(move |_, execute_context, cmd_buffer| {
                    let render_context = &execute_context.render_context;
                    let debug_renderpass = execute_context.debug_stuff.debug_renderpass;

                    let mesh_manager = execute_context.render_resources.get::<MeshManager>();
                    let model_manager = execute_context.render_resources.get::<ModelManager>();

                    debug_renderpass.render_ground_plane(render_context, cmd_buffer, &mesh_manager);

                    debug_renderpass.render_picked(
                        render_context,
                        cmd_buffer,
                        execute_context.debug_stuff.picked_drawables,
                        &mesh_manager,
                        &model_manager,
                    );

                    debug_renderpass.render_debug_display(
                        render_context,
                        cmd_buffer,
                        execute_context.debug_stuff.debug_display,
                        &mesh_manager,
                    );

                    debug_renderpass.render_manipulators(
                        render_context,
                        cmd_buffer,
                        RenderSurfaceExtents::new(
                            view_target_extents.width,
                            view_target_extents.height,
                        ),
                        execute_context.debug_stuff.manipulator_drawables,
                        &mesh_manager,
                        execute_context.debug_stuff.camera_component,
                    );
                })
        })
    }
}
