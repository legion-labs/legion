use lgn_graphics_api::{Format, GPUViewType};

use crate::core::{
    RenderGraphBuilder, RenderGraphLoadState, RenderGraphResourceDef, RenderGraphViewDef,
    RenderGraphViewId, RenderView,
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
            let raw_ao_buffer_desc = RenderGraphResourceDef::new_texture_2D(
                view_extents.width,
                view_extents.height,
                Format::R8_UNORM,
            );
            let raw_ao_buffer_id =
                builder.declare_render_target("AORawBuffer", &raw_ao_buffer_desc);
            let raw_ao_write_view_id =
                builder.declare_view(&RenderGraphViewDef::new_single_mip_texture_view(
                    raw_ao_buffer_id,
                    0,
                    GPUViewType::UnorderedAccess,
                ));
            let raw_ao_read_view_id =
                builder.declare_view(&RenderGraphViewDef::new_single_mip_texture_view(
                    raw_ao_buffer_id,
                    0,
                    GPUViewType::ShaderResource,
                ));
            let blur_buffer_id = builder.declare_render_target("AOBlurBuffer", &raw_ao_buffer_desc);
            let blur_write_view_id =
                builder.declare_view(&RenderGraphViewDef::new_single_mip_texture_view(
                    blur_buffer_id,
                    0,
                    GPUViewType::UnorderedAccess,
                ));
            let blur_read_view_id =
                builder.declare_view(&RenderGraphViewDef::new_single_mip_texture_view(
                    blur_buffer_id,
                    0,
                    GPUViewType::ShaderResource,
                ));

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
