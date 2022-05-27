use crate::core::{
    RenderGraphBuilder, RenderGraphContext, RenderGraphExecuteContext, RenderGraphLoadState,
    RenderGraphViewId,
};
use lgn_graphics_api::CommandBuffer;

pub struct PostProcessPass;

impl PostProcessPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        radiance_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        // Note this function does not specify the correct resources for passes, it's mostly to show
        // multiple nested scopes and passes at different levels.

        builder.add_scope("PostProcess", |builder| {
            builder
                .add_scope("DepthOfField", |builder| {
                    // This could be a separate struct DepthOfFieldPass with its own build_render_graph(builder) method.
                    builder
                        .add_compute_pass("DOF CoC", |compute_pass_builder| {
                            compute_pass_builder
                                .read(radiance_view_id, RenderGraphLoadState::Load)
                                .write(radiance_view_id, RenderGraphLoadState::Load)
                                .execute(Self::execute_dof_coc)
                        })
                        .add_compute_pass("DOF Blur CoC", |compute_pass_builder| {
                            compute_pass_builder
                                .read(radiance_view_id, RenderGraphLoadState::Load)
                                .write(radiance_view_id, RenderGraphLoadState::Load)
                                .execute(|_, _, _| {
                                    //println!("DOF Blur CoC pass execute");
                                })
                        })
                        .add_compute_pass("DOF Composite", |compute_pass_builder| {
                            compute_pass_builder
                                .read(radiance_view_id, RenderGraphLoadState::Load)
                                .write(radiance_view_id, RenderGraphLoadState::Load)
                                .execute(|_, _, _| {
                                    //println!("DOF Composite pass execute");
                                })
                        })
                })
                .add_scope("Bloom", |builder| {
                    // This could be a separate struct BloomPass with its own build_render_graph(builder) method.
                    builder
                        .add_compute_pass("Bloom Downsample", |compute_pass_builder| {
                            compute_pass_builder
                                .read(radiance_view_id, RenderGraphLoadState::Load)
                                .write(radiance_view_id, RenderGraphLoadState::Load)
                                .execute(|_, _, _| {
                                    //println!("Bloom Downsample pass execute");
                                })
                        })
                        .add_compute_pass("Bloom Threshold", |compute_pass_builder| {
                            compute_pass_builder
                                .read(radiance_view_id, RenderGraphLoadState::Load)
                                .write(radiance_view_id, RenderGraphLoadState::Load)
                                .execute(|_, _, _| {
                                    //println!("Bloom Threshold pass execute");
                                })
                        })
                        .add_compute_pass("Bloom Apply", |compute_pass_builder| {
                            compute_pass_builder
                                .read(radiance_view_id, RenderGraphLoadState::Load)
                                .write(radiance_view_id, RenderGraphLoadState::Load)
                                .execute(|_, _, _| {
                                    //println!("Bloom Apply pass execute");
                                })
                        })
                })
                // This could be a separate struct ToneMappingPass with its own build_render_graph(builder) method.
                .add_compute_pass("ToneMapping", |compute_pass_builder| {
                    compute_pass_builder
                        .read(radiance_view_id, RenderGraphLoadState::Load)
                        .write(radiance_view_id, RenderGraphLoadState::Load)
                        .execute(|_, _, _| {
                            //println!("ToneMapping pass execute");
                        })
                })
        })
    }

    fn execute_dof_coc(
        _context: &RenderGraphContext,
        _execute_context: &mut RenderGraphExecuteContext<'_, '_>,
        _cmd_buffer: &mut CommandBuffer,
    ) {
        //println!("DOF CoC pass execute");
    }
}
