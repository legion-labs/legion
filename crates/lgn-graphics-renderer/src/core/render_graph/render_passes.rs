use std::any::Any;

use lgn_graphics_api::{
    ColorClearValue, CommandBuffer, DepthStencilClearValue, Extents3D, Format, PlaneSlice,
    ViewDimension,
};

use crate::{
    core::render_graph::RenderGraphBuilder,
    core::render_graph::RenderView,
    core::render_graph::{
        RenderGraphExecuteContext, RenderGraphResourceDef, RenderGraphResourceId,
        RenderGraphTextureDef, RenderGraphTextureViewDef, RenderGraphViewDef, RenderGraphViewId,
    },
    gpu_renderer::DefaultLayers,
    resources::UnifiedStaticBuffer,
    RenderContext,
};

use super::RenderGraphLoadState;

pub struct GpuCullingPass;

pub struct DepthLayerPass;

pub struct OpaqueLayerPass;

pub struct AlphaBlendedLayerPass;

pub struct PostProcessPass;

pub struct LightingPass;

pub struct SSAOPass;

pub struct UiPass;

pub struct GPUCullingUserData {
    mip: u32,
}

// From https://github.com/DenisKolodin/match_cast
#[macro_export]
macro_rules! match_cast {
    ($any:ident { $( $bind:ident as $patt:ty => $body:block , )+ }) => {{
        let downcast = || {
            $(
            if let Some($bind) = $any.downcast_ref::<$patt>() {
                return Some($body);
            }
            )+
            None
        };
        downcast()
    }};
}

impl GpuCullingPass {
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderGraphResourceId,
        depth_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder {
        let depth_resource_def = builder.get_resource_def(depth_buffer_id);
        let depth_resource_def: &RenderGraphTextureDef = depth_resource_def.try_into().unwrap();
        let depth_buffer_extents = depth_resource_def.extents;
        let hzb_desc = self.make_hzb_desc(depth_buffer_extents);
        let mut builder = builder;
        let hzb_id = builder.declare_render_target("HZB", &hzb_desc);

        builder.add_scope("HZB", |builder| {
            self.build_hzb_render_graph(depth_buffer_id, depth_view_id, hzb_id, &hzb_desc, builder)
        })
    }

    #[allow(clippy::unused_self)]
    fn make_hzb_desc(&self, extents: Extents3D) -> RenderGraphResourceDef {
        const SCALE_THRESHOLD: f32 = 0.7;

        let mut hzb_width = 2.0f32.powf((extents.width as f32).log2().floor());
        if hzb_width / extents.width as f32 > SCALE_THRESHOLD {
            hzb_width /= 2.0;
        }
        let mut hzb_height = 2.0f32.powf((extents.height as f32).log2().floor());
        if hzb_height / extents.height as f32 > SCALE_THRESHOLD {
            hzb_height /= 2.0;
        }

        hzb_width = hzb_width.max(4.0);
        hzb_height = hzb_height.max(4.0);

        let mut min_extent = hzb_width.min(hzb_height) as u32;
        let mut mip_count = 1;
        while min_extent != 1 {
            min_extent /= 2;
            mip_count += 1;
        }

        RenderGraphResourceDef::Texture(RenderGraphTextureDef {
            extents: Extents3D {
                width: hzb_width as u32,
                height: hzb_height as u32,
                depth: 1,
            },
            array_length: 1,
            mip_count,
            format: Format::R32_SFLOAT,
        })
    }

    #[allow(clippy::unused_self)]
    fn build_hzb_render_graph(
        &self,
        depth_buffer_id: RenderGraphResourceId,
        _depth_view_id: RenderGraphViewId,
        hzb_id: RenderGraphResourceId,
        hzb_desc: &RenderGraphResourceDef,
        builder: RenderGraphBuilder,
    ) -> RenderGraphBuilder {
        let mut builder = builder;

        let depth_view_def = RenderGraphTextureViewDef {
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Default,
            first_array_slice: 0,
            array_size: 1,
            read_only: false,
        };

        let hzb_desc: &RenderGraphTextureDef = hzb_desc.try_into().unwrap();
        for i in 0..hzb_desc.mip_count {
            let read_res_id = if i == 0 { depth_buffer_id } else { hzb_id };
            let write_res_id = hzb_id;

            let mut read_view_def = depth_view_def.clone();
            let read_view_id = if i == 0 {
                read_view_def.plane_slice = PlaneSlice::Depth;
                builder.declare_view(&RenderGraphViewDef::Texture(read_view_def.clone()))
            } else {
                read_view_def.first_mip = i - 1;
                builder.declare_view(&RenderGraphViewDef::Texture(read_view_def.clone()))
            };

            let mut write_view_def = depth_view_def.clone();
            write_view_def.first_mip = i;
            write_view_def.plane_slice = PlaneSlice::Default;
            let write_view_id =
                builder.declare_view(&RenderGraphViewDef::Texture(write_view_def.clone()));

            let pass_name = format!("HZB mip {}", i);
            builder = builder.add_compute_pass(&pass_name, move |mut compute_pass_builder| {
                let user_data = GPUCullingUserData { mip: i };
                compute_pass_builder = compute_pass_builder
                    .read(read_res_id, read_view_id, RenderGraphLoadState::Load)
                    .write(write_res_id, write_view_id, RenderGraphLoadState::DontCare)
                    .execute_with_data(
                        |_, _, _, user_data| {
                            let user_data = user_data.as_ref().unwrap();
                            let user_data =
                                match_cast!(user_data { val as GPUCullingUserData => {val},})
                                    .unwrap();
                            let mip = user_data.mip;
                            println!("HZB execute mip {}", mip);
                        },
                        Box::new(user_data),
                    );

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
                    RenderGraphLoadState::ClearDepthStencil(DepthStencilClearValue {
                        depth: 1.0,
                        stencil: 0,
                    }),
                )
                .execute(Self::execute_depth_layer_pass)
        })
    }

    fn execute_depth_layer_pass(
        execute_context: &RenderGraphExecuteContext<'_>,
        render_context: &RenderContext<'_>,
        command_buffer: &mut CommandBuffer,
        _user_data: &Option<Box<dyn Any>>,
    ) {
        let static_buffer = execute_context
            .render_resources
            .get::<UnifiedStaticBuffer>();

        command_buffer.cmd_bind_index_buffer(static_buffer.index_buffer_binding());
        command_buffer.cmd_bind_vertex_buffers(
            0,
            &[execute_context
                .render_managers
                .instance_manager
                .vertex_buffer_binding()],
        );

        println!("DepthLayerPass execute");
        execute_context.render_managers.mesh_renderer.draw(
            render_context,
            command_buffer,
            DefaultLayers::Depth,
        );
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
                //                .render_target(
                //                    1,
                //                    gbuffer_ids[1],
                //                    gbuffer_view_id,
                //                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
                //                )
                //                .render_target(
                //                    2,
                //                    gbuffer_ids[2],
                //                    gbuffer_view_id,
                //                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
                //                )
                //                .render_target(
                //                    3,
                //                    gbuffer_ids[3],
                //                    gbuffer_view_id,
                //                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
                //                )
                .depth_stencil(depth_buffer_id, depth_view_id, RenderGraphLoadState::Load)
                .execute(Self::execute_opaque_layer_pass)
        })
    }

    fn execute_opaque_layer_pass(
        execute_context: &RenderGraphExecuteContext<'_>,
        render_context: &RenderContext<'_>,
        command_buffer: &mut CommandBuffer,
        _user_data: &Option<Box<dyn Any>>,
    ) {
        let static_buffer = execute_context
            .render_resources
            .get::<UnifiedStaticBuffer>();

        command_buffer.cmd_bind_index_buffer(static_buffer.index_buffer_binding());
        command_buffer.cmd_bind_vertex_buffers(
            0,
            &[execute_context
                .render_managers
                .instance_manager
                .vertex_buffer_binding()],
        );

        println!("OpaqueLayerPass execute");
        execute_context.render_managers.mesh_renderer.draw(
            render_context,
            command_buffer,
            DefaultLayers::Opaque,
        );
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
                .execute(|_, _, _, _| {
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
                                .execute(|_, _, _, _| {
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
                                .execute(|_, _, _, _| {
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
                                .execute(|_, _, _, _| {
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
                                .execute(|_, _, _, _| {
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
                                .execute(|_, _, _, _| {
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
                        .execute(|_, _, _, _| {
                            println!("ToneMapping pass execute");
                        })
                })
        })
    }

    fn execute_dof_coc(
        _execute_context: &RenderGraphExecuteContext<'_>,
        _render_context: &RenderContext<'_>,
        _command_buffer: &mut CommandBuffer,
        _user_data: &Option<Box<dyn Any>>,
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
                    RenderGraphLoadState::DontCare,
                )
                .execute(|_, _, _, _| {
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
                        .write(raw_ao_buffer_id, ao_view_id, RenderGraphLoadState::DontCare)
                        .execute(|_, _, _, _| {
                            println!("AO pass execute");
                        })
                })
                .add_compute_pass("BlurX", |compute_pass_builder| {
                    compute_pass_builder
                        .read(raw_ao_buffer_id, ao_view_id, RenderGraphLoadState::Load)
                        .read(depth_buffer_id, depth_view_id, RenderGraphLoadState::Load)
                        .write(blur_buffer_id, ao_view_id, RenderGraphLoadState::DontCare)
                        .execute(|_, _, _, _| {
                            println!("BlurX pass execute");
                        })
                })
                .add_compute_pass("BlurY", |compute_pass_builder| {
                    compute_pass_builder
                        .read(blur_buffer_id, ao_view_id, RenderGraphLoadState::Load)
                        .read(depth_buffer_id, depth_view_id, RenderGraphLoadState::Load)
                        .write(ao_buffer_id, ao_view_id, RenderGraphLoadState::DontCare)
                        .execute(|_, _, _, _| {
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
                .render_target(0, ui_buffer_id, ui_view_id, RenderGraphLoadState::DontCare)
                .execute(|_, _, _, _| {
                    println!("UiPass execute");
                })
        })
    }
}
