use lgn_graphics_api::{
    AddressMode, BlendState, ColorClearValue, CommandBuffer, CompareOp, CullMode, DepthState,
    DepthStencilClearValue, Extents3D, FilterType, Format, GPUViewType, GraphicsPipelineDef,
    MipMapMode, PlaneSlice, PrimitiveTopology, RasterizerState, ResourceUsage, SampleCount,
    Sampler, SamplerDef, StencilOp, VertexAttributeRate, VertexLayout, VertexLayoutAttribute,
    VertexLayoutBuffer, ViewDimension,
};
use lgn_graphics_cgen_runtime::CGenShaderKey;

use crate::{
    cgen,
    core::render_graph::RenderGraphBuilder,
    core::render_graph::RenderView,
    core::render_graph::{
        RenderGraphExecuteContext, RenderGraphResourceDef, RenderGraphResourceId,
        RenderGraphTextureDef, RenderGraphTextureViewDef, RenderGraphViewDef, RenderGraphViewId,
    },
    gpu_renderer::{DefaultLayers, GpuInstanceManager, MeshRenderer},
    resources::{PipelineDef, PipelineHandle, PipelineManager, UnifiedStaticBuffer},
    RenderContext,
};

use super::{RenderGraphContext, RenderGraphLoadState};

pub struct GpuCullingPass;

pub struct DepthLayerPass;

pub struct OpaqueLayerPass;

pub struct AlphaBlendedLayerPass;

pub struct PostProcessPass;

pub struct LightingPass;

pub struct SSAOPass;

pub struct UiPass;

#[derive(Clone)]
pub struct GPUCullingUserData {
    pipeline_handle: PipelineHandle,
    mip_sampler: Sampler,
}

pub struct GPUCullingMipUserData {
    base_user_data: GPUCullingUserData,
    mip: u32,
}

impl GpuCullingPass {
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        depth_buffer_id: RenderGraphResourceId,
        depth_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
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

    fn build_hzb_pso(pipeline_manager: &PipelineManager) -> PipelineHandle {
        let root_signature = cgen::pipeline_layout::HzbPipelineLayout::root_signature();

        let mut vertex_layout = VertexLayout::default();
        vertex_layout.attributes[0] = Some(VertexLayoutAttribute {
            format: Format::R32G32_SFLOAT,
            buffer_index: 0,
            location: 0,
            byte_offset: 0,
        });
        vertex_layout.attributes[1] = Some(VertexLayoutAttribute {
            format: Format::R32G32_SFLOAT,
            buffer_index: 0,
            location: 1,
            byte_offset: 8,
        });
        vertex_layout.buffers[0] = Some(VertexLayoutBuffer {
            stride: 16,
            rate: VertexAttributeRate::Vertex,
        });

        let depth_state = DepthState {
            depth_test_enable: false,
            depth_write_enable: false,
            depth_compare_op: CompareOp::Never,
            stencil_test_enable: false,
            stencil_read_mask: 0xFF,
            stencil_write_mask: 0xFF,
            front_depth_fail_op: StencilOp::default(),
            front_stencil_compare_op: CompareOp::default(),
            front_stencil_fail_op: StencilOp::default(),
            front_stencil_pass_op: StencilOp::default(),
            back_depth_fail_op: StencilOp::default(),
            back_stencil_compare_op: CompareOp::default(),
            back_stencil_fail_op: StencilOp::default(),
            back_stencil_pass_op: StencilOp::default(),
        };

        let rasterizer_state = RasterizerState {
            cull_mode: CullMode::Back,
            ..RasterizerState::default()
        };

        let shader = pipeline_manager
            .create_shader(
                cgen::CRATE_ID,
                CGenShaderKey::make(cgen::shader::hzb_shader::ID, cgen::shader::hzb_shader::NONE),
            )
            .unwrap();
        pipeline_manager.register_pipeline(PipelineDef::Graphics(GraphicsPipelineDef {
            shader,
            root_signature: root_signature.clone(),
            vertex_layout,
            blend_state: BlendState::default_alpha_disabled(),
            depth_state,
            rasterizer_state,
            color_formats: vec![Format::R32_SFLOAT],
            sample_count: SampleCount::SampleCount1,
            depth_stencil_format: None,
            primitive_topology: PrimitiveTopology::TriangleList,
        }))
    }

    #[allow(clippy::unused_self)]
    fn build_hzb_render_graph<'a>(
        &self,
        depth_buffer_id: RenderGraphResourceId,
        _depth_view_id: RenderGraphViewId,
        hzb_id: RenderGraphResourceId,
        hzb_desc: &RenderGraphResourceDef,
        builder: RenderGraphBuilder<'a>,
    ) -> RenderGraphBuilder<'a> {
        let mut builder = builder;

        let depth_view_def = RenderGraphTextureViewDef {
            resource_id: depth_buffer_id,
            gpu_view_type: lgn_graphics_api::GPUViewType::ShaderResource,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Depth,
            first_array_slice: 0,
            array_size: 1,
            read_only: false,
        };

        let hzb_desc: &RenderGraphTextureDef = hzb_desc.try_into().unwrap();

        let pipeline_handle = Self::build_hzb_pso(builder.pipeline_manager);
        let mip_sampler = builder.device_context.create_sampler(SamplerDef {
            min_filter: FilterType::Nearest,
            mag_filter: FilterType::Nearest,
            mip_map_mode: MipMapMode::Nearest,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mip_lod_bias: 0.0,
            max_anisotropy: 1.0,
            compare_op: CompareOp::Never,
        });

        let base_user_data = GPUCullingUserData {
            pipeline_handle,
            mip_sampler,
        };

        for i in 0..hzb_desc.mip_count {
            let mut read_view_def = depth_view_def.clone();
            let read_view_id = if i == 0 {
                builder.declare_view(&RenderGraphViewDef::Texture(read_view_def.clone()))
            } else {
                read_view_def.resource_id = hzb_id;
                read_view_def.plane_slice = PlaneSlice::Default;
                read_view_def.first_mip = i - 1;
                builder.declare_view(&RenderGraphViewDef::Texture(read_view_def.clone()))
            };

            let mut write_view_def = depth_view_def.clone();
            write_view_def.resource_id = hzb_id;
            write_view_def.gpu_view_type = GPUViewType::RenderTarget;
            write_view_def.first_mip = i;
            write_view_def.plane_slice = PlaneSlice::Default;
            let write_view_id =
                builder.declare_view(&RenderGraphViewDef::Texture(write_view_def.clone()));

            let pass_name = format!("HZB mip {}", i);
            builder = builder.add_graphics_pass(&pass_name, |mut compute_pass_builder| {
                let user_data = GPUCullingMipUserData {
                    mip: i,
                    base_user_data: base_user_data.clone(),
                };

                compute_pass_builder = compute_pass_builder
                    .read(read_view_id, RenderGraphLoadState::Load)
                    .render_target(0, write_view_id, RenderGraphLoadState::DontCare)
                    .execute(move |context, execute_context, command_buffer| {
                        let render_context: &mut RenderContext<'_> = execute_context.render_context;

                        let read_view = context.get_texture_view(read_view_id);

                        if let Some(pipeline) = render_context
                            .pipeline_manager
                            .get_pipeline(user_data.base_user_data.pipeline_handle)
                        {
                            command_buffer.cmd_bind_pipeline(pipeline);

                            let mut descriptor_set =
                                cgen::descriptor_set::HzbDescriptorSet::default();
                            descriptor_set.set_depth_texture(read_view);
                            descriptor_set.set_depth_sampler(&user_data.base_user_data.mip_sampler);

                            let descriptor_set_handle = render_context.write_descriptor_set(
                                cgen::descriptor_set::HzbDescriptorSet::descriptor_set_layout(),
                                descriptor_set.descriptor_refs(),
                            );
                            command_buffer.cmd_bind_descriptor_set_handle(
                                cgen::descriptor_set::HzbDescriptorSet::descriptor_set_layout(),
                                descriptor_set_handle,
                            );

                            #[rustfmt::skip]
                                let vertex_data: [f32; 12] = [0.0, 2.0, 0.0, 2.0,
                                                              0.0, 0.0, 0.0, 0.0,
                                                              2.0, 0.0, 2.0, 0.0];

                            let transient_buffer = render_context
                                .transient_buffer_allocator
                                .copy_data_slice(&vertex_data, ResourceUsage::AS_VERTEX_BUFFER);

                            let vertex_binding = transient_buffer.vertex_buffer_binding();

                            command_buffer.cmd_bind_vertex_buffer(0, vertex_binding);

                            command_buffer.cmd_draw(3, 0);

                            let mip = user_data.mip;
                            println!("HZB execute mip {}", mip);
                        }
                    });

                compute_pass_builder
            });
        }
        builder
    }
}

impl DepthLayerPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        depth_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        builder.add_graphics_pass("DepthLayer", |graphics_pass_builder| {
            graphics_pass_builder
                .depth_stencil(
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
        _context: &RenderGraphContext,
        execute_context: &mut RenderGraphExecuteContext<'_, '_>,
        command_buffer: &mut CommandBuffer,
    ) {
        let render_context = &execute_context.render_context;

        let static_buffer = execute_context
            .render_resources
            .get::<UnifiedStaticBuffer>();

        command_buffer.cmd_bind_index_buffer(static_buffer.index_buffer_binding());
        command_buffer.cmd_bind_vertex_buffers(
            0,
            &[execute_context
                .render_resources
                .get::<GpuInstanceManager>()
                .vertex_buffer_binding()],
        );

        println!("DepthLayerPass execute");
        execute_context.render_resources.get::<MeshRenderer>().draw(
            render_context,
            command_buffer,
            DefaultLayers::Depth,
        );
    }
}

impl OpaqueLayerPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        depth_view_id: RenderGraphViewId,
        gbuffer_view_ids: [RenderGraphViewId; 4],
    ) -> RenderGraphBuilder<'a> {
        builder.add_graphics_pass("OpaqueLayer", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(
                    0,
                    gbuffer_view_ids[0],
                    RenderGraphLoadState::ClearColor(ColorClearValue([0.0; 4])),
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
                .execute(Self::execute_opaque_layer_pass)
        })
    }

    fn execute_opaque_layer_pass(
        _context: &RenderGraphContext,
        execute_context: &mut RenderGraphExecuteContext<'_, '_>,
        command_buffer: &mut CommandBuffer,
    ) {
        let render_context = &execute_context.render_context;

        let static_buffer = execute_context
            .render_resources
            .get::<UnifiedStaticBuffer>();

        command_buffer.cmd_bind_index_buffer(static_buffer.index_buffer_binding());
        command_buffer.cmd_bind_vertex_buffers(
            0,
            &[execute_context
                .render_resources
                .get::<GpuInstanceManager>()
                .vertex_buffer_binding()],
        );

        println!("OpaqueLayerPass execute");
        execute_context.render_resources.get::<MeshRenderer>().draw(
            render_context,
            command_buffer,
            DefaultLayers::Opaque,
        );
    }
}

impl AlphaBlendedLayerPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        depth_view_id: RenderGraphViewId,
        radiance_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        builder.add_graphics_pass("AlphaBlendedLayer", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(0, radiance_view_id, RenderGraphLoadState::Load)
                .depth_stencil(depth_view_id, RenderGraphLoadState::Load)
                .execute(|_, _, _| {
                    println!("AlphaBlendedLayerPass execute");
                })
        })
    }
}

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
                                    println!("DOF Blur CoC pass execute");
                                })
                        })
                        .add_compute_pass("DOF Composite", |compute_pass_builder| {
                            compute_pass_builder
                                .read(radiance_view_id, RenderGraphLoadState::Load)
                                .write(radiance_view_id, RenderGraphLoadState::Load)
                                .execute(|_, _, _| {
                                    println!("DOF Composite pass execute");
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
                                    println!("Bloom Downsample pass execute");
                                })
                        })
                        .add_compute_pass("Bloom Threshold", |compute_pass_builder| {
                            compute_pass_builder
                                .read(radiance_view_id, RenderGraphLoadState::Load)
                                .write(radiance_view_id, RenderGraphLoadState::Load)
                                .execute(|_, _, _| {
                                    println!("Bloom Threshold pass execute");
                                })
                        })
                        .add_compute_pass("Bloom Apply", |compute_pass_builder| {
                            compute_pass_builder
                                .read(radiance_view_id, RenderGraphLoadState::Load)
                                .write(radiance_view_id, RenderGraphLoadState::Load)
                                .execute(|_, _, _| {
                                    println!("Bloom Apply pass execute");
                                })
                        })
                })
                // This could be a separate struct ToneMappingPass with its own build_render_graph(builder) method.
                .add_compute_pass("ToneMapping", |compute_pass_builder| {
                    compute_pass_builder
                        .read(radiance_view_id, RenderGraphLoadState::Load)
                        .write(radiance_view_id, RenderGraphLoadState::Load)
                        .execute(|_, _, _| {
                            println!("ToneMapping pass execute");
                        })
                })
        })
    }

    fn execute_dof_coc(
        _context: &RenderGraphContext,
        _execute_context: &mut RenderGraphExecuteContext<'_, '_>,
        _command_buffer: &mut CommandBuffer,
    ) {
        println!("DOF CoC pass execute");
    }
}

impl LightingPass {
    #[allow(clippy::unused_self)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        depth_view_id: RenderGraphViewId,
        gbuffer_view_ids: [RenderGraphViewId; 4],
        ao_view_id: RenderGraphViewId,
        radiance_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        builder.add_compute_pass("Lighting", |compute_pass_builder| {
            compute_pass_builder
                .read(gbuffer_view_ids[0], RenderGraphLoadState::Load)
                .read(gbuffer_view_ids[1], RenderGraphLoadState::Load)
                .read(gbuffer_view_ids[2], RenderGraphLoadState::Load)
                .read(gbuffer_view_ids[3], RenderGraphLoadState::Load)
                .read(depth_view_id, RenderGraphLoadState::Load)
                .read(ao_view_id, RenderGraphLoadState::Load)
                .write(radiance_view_id, RenderGraphLoadState::DontCare)
                .execute(|_, _, _| {
                    println!("LightingPass execute");
                })
        })
    }
}

impl SSAOPass {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        view: &RenderView,
        depth_view_id: RenderGraphViewId,
        gbuffer_view_ids: [RenderGraphViewId; 4],
        ao_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        builder.add_scope("SSAO", |builder| {
            let raw_ao_buffer_desc = self.make_raw_ao_buffer_desc(view);
            let mut builder = builder;
            let raw_ao_buffer_id =
                builder.declare_render_target("AORawBuffer", &raw_ao_buffer_desc);
            let raw_ao_write_view_id = builder
                .declare_view(&self.make_single_mip_color_write_uav_view_def(raw_ao_buffer_id));
            let raw_ao_read_view_id =
                builder.declare_view(&self.make_single_mip_color_view_def(raw_ao_buffer_id));
            let blur_buffer_id = builder.declare_render_target("AOBlurBuffer", &raw_ao_buffer_desc);
            let blur_write_view_id = builder
                .declare_view(&self.make_single_mip_color_write_uav_view_def(blur_buffer_id));
            let blur_read_view_id =
                builder.declare_view(&self.make_single_mip_color_view_def(blur_buffer_id));

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
                            println!("AO pass execute");
                        })
                })
                .add_compute_pass("BlurX", |compute_pass_builder| {
                    compute_pass_builder
                        .read(raw_ao_read_view_id, RenderGraphLoadState::Load)
                        .read(depth_view_id, RenderGraphLoadState::Load)
                        .write(blur_write_view_id, RenderGraphLoadState::DontCare)
                        .execute(|_, _, _| {
                            println!("BlurX pass execute");
                        })
                })
                .add_compute_pass("BlurY", |compute_pass_builder| {
                    compute_pass_builder
                        .read(blur_read_view_id, RenderGraphLoadState::Load)
                        .read(depth_view_id, RenderGraphLoadState::Load)
                        .write(ao_view_id, RenderGraphLoadState::DontCare)
                        .execute(|_, _, _| {
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

    #[allow(clippy::unused_self)]
    fn make_single_mip_color_view_def(
        &self,
        resource_id: RenderGraphResourceId,
    ) -> RenderGraphViewDef {
        RenderGraphViewDef::Texture(RenderGraphTextureViewDef {
            resource_id,
            gpu_view_type: GPUViewType::ShaderResource,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Default,
            first_array_slice: 0,
            array_size: 1,
            read_only: false,
        })
    }

    #[allow(clippy::unused_self)]
    fn make_single_mip_color_write_uav_view_def(
        &self,
        resource_id: RenderGraphResourceId,
    ) -> RenderGraphViewDef {
        RenderGraphViewDef::Texture(RenderGraphTextureViewDef {
            resource_id,
            gpu_view_type: GPUViewType::UnorderedAccess,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Default,
            first_array_slice: 0,
            array_size: 1,
            read_only: false,
        })
    }
}

impl UiPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph<'a>(
        &self,
        builder: RenderGraphBuilder<'a>,
        ui_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder<'a> {
        builder.add_graphics_pass("UI", |graphics_pass_builder| {
            graphics_pass_builder
                .render_target(0, ui_view_id, RenderGraphLoadState::DontCare)
                .execute(|_, _, _| {
                    println!("UiPass execute");
                })
        })
    }
}
