use lgn_graphics_api::{
    ColorClearValue, DepthStencilClearValue, Extents3D, Format, PlaneSlice, ViewDimension,
};

use crate::{
    hl_gfx_api::HLCommandBuffer,
    render_graph::RenderGraphBuilder,
    render_graph::RenderView,
    render_graph::{
        RenderGraphExecuteContext, RenderGraphResourceDef, RenderGraphResourceId,
        RenderGraphTextureDef, RenderGraphTextureViewDef, RenderGraphViewDef, RenderGraphViewId,
    },
};

use super::RenderGraphLoadState;

pub struct GpuCullingPass {}

pub struct DepthLayerPass {}

pub struct OpaqueLayerPass {}

pub struct AlphaBlendedLayerPass {}

pub struct PostProcessPass {}

pub struct LightingPass {}

pub struct SSAOPass {}

pub struct UiPass {}

impl GpuCullingPass {
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderGraphResourceId,
        depth_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder {
        let depth_buffer_extents = builder
            .get_resource_def(depth_buffer_id)
            .texture_def()
            .extents;
        let downsampled_depth_desc = self.make_downsampled_depth_desc(depth_buffer_extents);
        let mut builder = builder;
        let downsampled_depth_id =
            builder.declare_render_target("DownsampledDepth", &downsampled_depth_desc);

        builder.add_scope("DepthDownsample", |builder| {
            self.build_downsample_render_graph(
                depth_buffer_id,
                depth_view_id,
                downsampled_depth_id,
                &downsampled_depth_desc,
                builder,
            )
        })
    }

    #[allow(clippy::unused_self)]
    fn make_downsampled_depth_desc(&self, view_extents: Extents3D) -> RenderGraphResourceDef {
        let mut extents = view_extents;
        let mut mips = 1;
        while extents.width != 1 && extents.height != 1 {
            extents.width >>= 1;
            extents.height >>= 1;
            mips += 1;
        }

        RenderGraphResourceDef::Texture(RenderGraphTextureDef {
            extents: view_extents,
            array_length: 1,
            mip_count: mips,
            format: Format::R16G16_SFLOAT,
        })
    }

    #[allow(clippy::unused_self)]
    fn build_downsample_render_graph(
        &self,
        depth_buffer_id: RenderGraphResourceId,
        _depth_view_id: RenderGraphViewId,
        downsampled_depth_id: RenderGraphResourceId,
        downsampled_depth_desc: &RenderGraphResourceDef,
        builder: RenderGraphBuilder,
    ) -> RenderGraphBuilder {
        let mut builder = builder;

        let depth_view_def = RenderGraphViewDef::Texture(RenderGraphTextureViewDef {
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Default,
            first_array_slice: 0,
            array_size: 1,
            read_only: false,
        });

        for i in 0..downsampled_depth_desc.texture_def().mip_count {
            let read_res_id = if i == 0 {
                depth_buffer_id
            } else {
                downsampled_depth_id
            };
            let write_res_id = downsampled_depth_id;

            let mut read_view_def = depth_view_def.clone();
            let read_view_id = if i == 0 {
                read_view_def.texture_view_def_mut().plane_slice = PlaneSlice::Depth;
                builder.declare_view(&read_view_def)
            } else {
                read_view_def.texture_view_def_mut().first_mip = i - 1;
                builder.declare_view(&read_view_def)
            };

            let mut write_view_def = depth_view_def.clone();
            write_view_def.texture_view_def_mut().first_mip = i;
            write_view_def.texture_view_def_mut().plane_slice = PlaneSlice::Default;
            let write_view_id = builder.declare_view(&write_view_def);

            let pass_name = format!("DepthDownsample mip {}", i);
            let mip_index = i;
            builder = builder.add_compute_pass(&pass_name, move |mut compute_pass_builder| {
                // The mip 0 pass should be a straight copy, not a compute shader.
                compute_pass_builder = compute_pass_builder
                    .read(read_res_id, read_view_id, RenderGraphLoadState::Load)
                    .write(
                        write_res_id,
                        write_view_id,
                        if i == 0 {
                            RenderGraphLoadState::ClearValue(0)
                        } else {
                            RenderGraphLoadState::Load
                        },
                    )
                    .execute(move |_, _| println!("DepthDownsample execute mip {}", mip_index));

                compute_pass_builder
            });
        }
        builder
    }
}

impl DepthLayerPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderGraphResourceId,
        depth_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder {
        builder.add_graphics_pass("DepthLayer", |graphics_pass_builder| {
            graphics_pass_builder
                .depth_stencil(
                    depth_buffer_id,
                    depth_view_id,
                    RenderGraphLoadState::ClearDepth(DepthStencilClearValue {
                        depth: 1.0,
                        stencil: 0,
                    }),
                )
                .execute(Self::execute_depth_layer_pass)
        })
    }

    fn execute_depth_layer_pass(
        _execute_context: &RenderGraphExecuteContext<'_>,
        _command_buffer: &mut HLCommandBuffer<'_>,
    ) {
        println!("DepthLayerPass execute");
    }
}

impl OpaqueLayerPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderGraphResourceId,
        depth_view_id: RenderGraphViewId,
        gbuffer_ids: [RenderGraphResourceId; 4],
        gbuffer_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder {
        builder.add_graphics_pass("OpaqueLayer", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(
                    0,
                    gbuffer_ids[0],
                    gbuffer_view_id,
                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
                )
                .render_target(
                    1,
                    gbuffer_ids[1],
                    gbuffer_view_id,
                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
                )
                .render_target(
                    2,
                    gbuffer_ids[2],
                    gbuffer_view_id,
                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
                )
                .render_target(
                    3,
                    gbuffer_ids[3],
                    gbuffer_view_id,
                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
                )
                .depth_stencil(depth_buffer_id, depth_view_id, RenderGraphLoadState::Load)
                .execute(Self::execute_opaque_layer_pass)
        })
    }

    fn execute_opaque_layer_pass(
        _execute_context: &RenderGraphExecuteContext<'_>,
        _command_buffer: &mut HLCommandBuffer<'_>,
    ) {
        println!("OpaqueLayerPass execute");
    }
}

impl AlphaBlendedLayerPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderGraphResourceId,
        depth_view_id: RenderGraphViewId,
        radiance_buffer_id: RenderGraphResourceId,
        radiance_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder {
        builder.add_graphics_pass("AlphaBlendedLayer", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(
                    0,
                    radiance_buffer_id,
                    radiance_view_id,
                    RenderGraphLoadState::Load,
                )
                .depth_stencil(depth_buffer_id, depth_view_id, RenderGraphLoadState::Load)
                .execute(|_, _| {
                    println!("AlphaBlendedLayerPass execute");
                })
        })
    }
}

impl PostProcessPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        radiance_buffer_id: RenderGraphResourceId,
        radiance_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder {
        // Note this function does not specify the correct resources for passes, it's mostly to show
        // multiple nested scopes and passes at different levels.

        builder.add_scope("PostProcess", |builder| {
            builder
                .add_scope("DepthOfField", |builder| {
                    // This could be a separate struct DepthOfFieldPass with its own build_render_graph(builder) method.
                    builder
                        .add_compute_pass("DOF CoC", |compute_pass_builder| {
                            compute_pass_builder
                                .read(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .write(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .execute(Self::execute_dof_coc)
                        })
                        .add_compute_pass("DOF Blur CoC", |compute_pass_builder| {
                            compute_pass_builder
                                .read(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .write(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .execute(|_, _| {
                                    println!("DOF Blur CoC pass execute");
                                })
                        })
                        .add_compute_pass("DOF Composite", |compute_pass_builder| {
                            compute_pass_builder
                                .read(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .write(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .execute(|_, _| {
                                    println!("DOF Composite pass execute");
                                })
                        })
                })
                .add_scope("Bloom", |builder| {
                    // This could be a separate struct BloomPass with its own build_render_graph(builder) method.
                    builder
                        .add_compute_pass("Bloom Downsample", |compute_pass_builder| {
                            compute_pass_builder
                                .read(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .write(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .execute(|_, _| {
                                    println!("Bloom Downsample pass execute");
                                })
                        })
                        .add_compute_pass("Bloom Threshold", |compute_pass_builder| {
                            compute_pass_builder
                                .read(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .write(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .execute(|_, _| {
                                    println!("Bloom Threshold pass execute");
                                })
                        })
                        .add_compute_pass("Bloom Apply", |compute_pass_builder| {
                            compute_pass_builder
                                .read(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .write(
                                    radiance_buffer_id,
                                    radiance_view_id,
                                    RenderGraphLoadState::Load,
                                )
                                .execute(|_, _| {
                                    println!("Bloom Apply pass execute");
                                })
                        })
                })
                // This could be a separate struct ToneMappingPass with its own build_render_graph(builder) method.
                .add_compute_pass("ToneMapping", |compute_pass_builder| {
                    compute_pass_builder
                        .read(
                            radiance_buffer_id,
                            radiance_view_id,
                            RenderGraphLoadState::Load,
                        )
                        .write(
                            radiance_buffer_id,
                            radiance_view_id,
                            RenderGraphLoadState::Load,
                        )
                        .execute(|_, _| {
                            println!("ToneMapping pass execute");
                        })
                })
        })
    }

    fn execute_dof_coc(
        _execute_context: &RenderGraphExecuteContext<'_>,
        _command_buffer: &mut HLCommandBuffer<'_>,
    ) {
        println!("DOF CoC pass execute");
    }
}

impl LightingPass {
    #[allow(clippy::unused_self)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderGraphResourceId,
        depth_view_id: RenderGraphViewId,
        gbuffer_ids: [RenderGraphResourceId; 4],
        gbuffer_view_id: RenderGraphViewId,
        ao_buffer_id: RenderGraphResourceId,
        ao_view_id: RenderGraphViewId,
        radiance_buffer_id: RenderGraphResourceId,
        radiance_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder {
        builder.add_compute_pass("Lighting", |compute_pass_builder| {
            compute_pass_builder
                .read(gbuffer_ids[0], gbuffer_view_id, RenderGraphLoadState::Load)
                .read(gbuffer_ids[1], gbuffer_view_id, RenderGraphLoadState::Load)
                .read(gbuffer_ids[2], gbuffer_view_id, RenderGraphLoadState::Load)
                .read(gbuffer_ids[3], gbuffer_view_id, RenderGraphLoadState::Load)
                .read(depth_buffer_id, depth_view_id, RenderGraphLoadState::Load)
                .read(ao_buffer_id, ao_view_id, RenderGraphLoadState::Load)
                .write(
                    radiance_buffer_id,
                    radiance_view_id,
                    RenderGraphLoadState::ClearValue(0),
                )
                .execute(|_, _| {
                    println!("LightingPass execute");
                })
        })
    }
}

impl SSAOPass {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        view: &RenderView,
        depth_buffer_id: RenderGraphResourceId,
        depth_view_id: RenderGraphViewId,
        gbuffer_ids: [RenderGraphResourceId; 4],
        gbuffer_view_id: RenderGraphViewId,
        ao_buffer_id: RenderGraphResourceId,
        ao_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder {
        builder.add_scope("SSAO", |builder| {
            let raw_ao_buffer_desc = self.make_raw_ao_buffer_desc(view);
            let mut builder = builder;
            let raw_ao_buffer_id =
                builder.declare_render_target("AORawBuffer", &raw_ao_buffer_desc);
            let blur_buffer_id = builder.declare_render_target("AOBlurBuffer", &raw_ao_buffer_desc);

            builder
                .add_compute_pass("AO", |compute_pass_builder| {
                    compute_pass_builder
                        .read(gbuffer_ids[0], gbuffer_view_id, RenderGraphLoadState::Load)
                        .read(gbuffer_ids[1], gbuffer_view_id, RenderGraphLoadState::Load)
                        .read(gbuffer_ids[2], gbuffer_view_id, RenderGraphLoadState::Load)
                        .read(gbuffer_ids[3], gbuffer_view_id, RenderGraphLoadState::Load)
                        .read(depth_buffer_id, depth_view_id, RenderGraphLoadState::Load)
                        .write(
                            raw_ao_buffer_id,
                            ao_view_id,
                            RenderGraphLoadState::ClearValue(0),
                        )
                        .execute(|_, _| {
                            println!("AO pass execute");
                        })
                })
                .add_compute_pass("BlurX", |compute_pass_builder| {
                    compute_pass_builder
                        .read(raw_ao_buffer_id, ao_view_id, RenderGraphLoadState::Load)
                        .read(depth_buffer_id, depth_view_id, RenderGraphLoadState::Load)
                        .write(
                            blur_buffer_id,
                            ao_view_id,
                            RenderGraphLoadState::ClearValue(0),
                        )
                        .execute(|_, _| {
                            println!("BlurX pass execute");
                        })
                })
                .add_compute_pass("BlurY", |compute_pass_builder| {
                    compute_pass_builder
                        .read(blur_buffer_id, ao_view_id, RenderGraphLoadState::Load)
                        .read(depth_buffer_id, depth_view_id, RenderGraphLoadState::Load)
                        .write(
                            ao_buffer_id,
                            ao_view_id,
                            RenderGraphLoadState::ClearValue(0),
                        )
                        .execute(|_, _| {
                            println!("BlurY pass execute");
                        })
                })
        })
    }

    #[allow(clippy::unused_self)]
    fn make_raw_ao_buffer_desc(&self, view: &RenderView) -> RenderGraphResourceDef {
        RenderGraphResourceDef::Texture(RenderGraphTextureDef {
            extents: view.target.definition().extents,
            array_length: 1,
            mip_count: 1,
            format: Format::R8_UNORM,
        })
    }
}

impl UiPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        ui_buffer_id: RenderGraphResourceId,
        ui_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder {
        builder.add_graphics_pass("UI", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(
                    0,
                    ui_buffer_id,
                    ui_view_id,
                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
                )
                .execute(|_, _| {
                    println!("UiPass execute");
                })
        })
    }
}
