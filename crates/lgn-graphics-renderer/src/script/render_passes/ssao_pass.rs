use lgn_graphics_api::Format;

use crate::{
    core::{RenderGraphBuilder, RenderGraphLoadState, RenderGraphViewId},
    script::RenderView,
};

pub struct SSAOPass;

impl SSAOPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        view: &RenderView<'_>,
        depth_view_id: RenderGraphViewId,
        gbuffer_view_ids: [RenderGraphViewId; 4],
        ao_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        builder.add_scope("SSAO", |mut builder| {
            let view_extents = view.target.definition().extents;
            let raw_ao_buffer_id = builder.declare_single_mip_render_target(
                "AORawBuffer",
                view_extents.width,
                view_extents.height,
                Format::R8_UNORM,
            );
            let raw_ao_write_view_id = builder.declare_single_mip_texture_uav(raw_ao_buffer_id);
            let raw_ao_read_view_id = builder.declare_single_mip_texture_srv(raw_ao_buffer_id);
            let blur_buffer_id = builder.declare_single_mip_render_target(
                "AOBlurBuffer",
                view_extents.width,
                view_extents.height,
                Format::R8_UNORM,
            );
            let blur_write_view_id = builder.declare_single_mip_texture_uav(blur_buffer_id);
            let blur_read_view_id = builder.declare_single_mip_texture_srv(blur_buffer_id);

            builder
                .add_compute_pass("AO", |compute_pass_builder| {
                    compute_pass_builder
                        .read(gbuffer_view_ids[0], RenderGraphLoadState::Load)
                        .read(gbuffer_view_ids[1], RenderGraphLoadState::Load)
                        .read(gbuffer_view_ids[2], RenderGraphLoadState::Load)
                        .read(gbuffer_view_ids[3], RenderGraphLoadState::Load)
                        .read(depth_view_id, RenderGraphLoadState::Load)
                        .write(raw_ao_write_view_id, RenderGraphLoadState::DontCare)
                        .execute(|_, _, _| {
                            //println!("AO pass execute");
                        })
                })
                .add_compute_pass("BlurX", |compute_pass_builder| {
                    compute_pass_builder
                        .read(raw_ao_read_view_id, RenderGraphLoadState::Load)
                        .read(depth_view_id, RenderGraphLoadState::Load)
                        .write(blur_write_view_id, RenderGraphLoadState::DontCare)
                        .execute(|_, _, _| {
                            //println!("BlurX pass execute");
                        })
                })
                .add_compute_pass("BlurY", |compute_pass_builder| {
                    compute_pass_builder
                        .read(blur_read_view_id, RenderGraphLoadState::Load)
                        .read(depth_view_id, RenderGraphLoadState::Load)
                        .write(ao_view_id, RenderGraphLoadState::DontCare)
                        .execute(|_, _, _| {
                            //println!("BlurY pass execute");
                        })
                })
        })
    }
}
