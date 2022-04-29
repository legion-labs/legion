use crate::render_script::{
    Format, RenderGraphBuilder, RenderGraphExecuteContext, RenderTargetDesc, RenderTargetId,
    RenderView,
};

pub struct GpuCullingPass {}

pub struct DepthLayerPass {}

pub struct OpaqueLayerPass {}

pub struct AlphaBlendedLayerPass {}

pub struct PostProcessPass {}

pub struct LightingPass {}

pub struct SSAOPass {}

pub struct UiPass {}

//impl GpuCullingPass {
//    pub(crate) fn build_render_graph(&self, builder: RenderGraphBuilder) -> RenderGraphBuilder {
//        builder
//    }
//}

impl DepthLayerPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderTargetId,
    ) -> RenderGraphBuilder {
        builder.add_graphics_pass("DepthLayer", |graphics_pass_builder| {
            graphics_pass_builder
                .add_depth_stencil((depth_buffer_id, 0))
                .execute(Box::new(Self::execute_depth_layer_pass))
        })
    }

    fn execute_depth_layer_pass(execute_context: &RenderGraphExecuteContext) {
        println!("DepthLayerPass execute {}", execute_context.name);
    }
}

impl OpaqueLayerPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderTargetId,
        gbuffer_ids: [RenderTargetId; 4],
    ) -> RenderGraphBuilder {
        builder.add_graphics_pass("OpaqueLayer", |graphics_pass_builder| {
            graphics_pass_builder
                .add_render_target((gbuffer_ids[0], 0))
                .add_render_target((gbuffer_ids[1], 0))
                .add_render_target((gbuffer_ids[2], 0))
                .add_render_target((gbuffer_ids[3], 0))
                .add_depth_stencil((depth_buffer_id, 0))
                .execute(Box::new(Self::execute_opaque_layer_pass))
        })
    }

    fn execute_opaque_layer_pass(execute_context: &RenderGraphExecuteContext) {
        println!("OpaqueLayerPass execute {}", execute_context.name);
    }
}

impl AlphaBlendedLayerPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderTargetId,
        radiance_buffer_id: RenderTargetId,
    ) -> RenderGraphBuilder {
        builder.add_graphics_pass("AlphaBlendedLayer", |graphics_pass_builder| {
            graphics_pass_builder
                .add_render_target((radiance_buffer_id, 0))
                .add_depth_stencil((depth_buffer_id, 0))
                .execute(Box::new(|_| {
                    println!("AlphaBlendedLayerPass execute");
                }))
        })
    }
}

impl PostProcessPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        radiance_buffer_id: RenderTargetId,
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
                                .add_read_resource((radiance_buffer_id, 0))
                                .add_write_resource((radiance_buffer_id, 0))
                                .execute(Box::new(Self::execute_dof_coc))
                        })
                        .add_compute_pass("DOF Blur CoC", |compute_pass_builder| {
                            compute_pass_builder
                                .add_read_resource((radiance_buffer_id, 0))
                                .add_write_resource((radiance_buffer_id, 0))
                                .execute(Box::new(|_| {
                                    println!("DOF Blur CoC pass execute");
                                }))
                        })
                        .add_compute_pass("DOF Composite", |compute_pass_builder| {
                            compute_pass_builder
                                .add_read_resource((radiance_buffer_id, 0))
                                .add_write_resource((radiance_buffer_id, 0))
                                .execute(Box::new(|_| {
                                    println!("DOF Composite pass execute");
                                }))
                        })
                })
                .add_scope("Bloom", |builder| {
                    // This could be a separate struct BloomPass with its own build_render_graph(builder) method.
                    builder
                        .add_compute_pass("Bloom Downsample", |compute_pass_builder| {
                            compute_pass_builder
                                .add_read_resource((radiance_buffer_id, 0))
                                .add_write_resource((radiance_buffer_id, 0))
                                .execute(Box::new(|_| {
                                    println!("Bloom Downsample pass execute");
                                }))
                        })
                        .add_compute_pass("Bloom Threshold", |compute_pass_builder| {
                            compute_pass_builder
                                .add_read_resource((radiance_buffer_id, 0))
                                .add_write_resource((radiance_buffer_id, 0))
                                .execute(Box::new(|_| {
                                    println!("Bloom Threshold pass execute");
                                }))
                        })
                        .add_compute_pass("Bloom Apply", |compute_pass_builder| {
                            compute_pass_builder
                                .add_read_resource((radiance_buffer_id, 0))
                                .add_write_resource((radiance_buffer_id, 0))
                                .execute(Box::new(|_| {
                                    println!("Bloom Apply pass execute");
                                }))
                        })
                })
                // This could be a separate struct ToneMappingPass with its own build_render_graph(builder) method.
                .add_compute_pass("ToneMapping", |compute_pass_builder| {
                    compute_pass_builder
                        .add_read_resource((radiance_buffer_id, 0))
                        .add_write_resource((radiance_buffer_id, 0))
                        .execute(Box::new(|_| {
                            println!("ToneMapping pass execute");
                        }))
                })
        })
    }

    fn execute_dof_coc(execute_context: &RenderGraphExecuteContext) {
        println!("DOF CoC pass execute {}", execute_context.name);
    }
}

impl LightingPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderTargetId,
        gbuffer_ids: [RenderTargetId; 4],
        ao_buffer_id: RenderTargetId,
        radiance_buffer_id: RenderTargetId,
    ) -> RenderGraphBuilder {
        builder.add_compute_pass("Lighting", |compute_pass_builder| {
            compute_pass_builder
                .add_read_resource((gbuffer_ids[0], 0))
                .add_read_resource((gbuffer_ids[1], 0))
                .add_read_resource((gbuffer_ids[2], 0))
                .add_read_resource((gbuffer_ids[3], 0))
                .add_read_resource((depth_buffer_id, 0))
                .add_read_resource((ao_buffer_id, 0))
                .add_write_resource((radiance_buffer_id, 0))
                .execute(Box::new(|_| {
                    println!("LightingPass execute");
                }))
        })
    }
}

impl SSAOPass {
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        view: &RenderView,
        depth_buffer_id: RenderTargetId,
        gbuffer_ids: [RenderTargetId; 4],
        ao_buffer_id: RenderTargetId,
    ) -> RenderGraphBuilder {
        builder.add_scope("SSAO", |builder| {
            let mut raw_ao_buffer_desc = self.make_raw_ao_buffer_desc(view);
            let mut builder = builder;
            let raw_ao_buffer_id = builder.declare_render_target(&raw_ao_buffer_desc);

            raw_ao_buffer_desc.name = "AOBlurBuffer".to_string();
            let blur_buffer_id = builder.declare_render_target(&raw_ao_buffer_desc);

            builder
                .add_compute_pass("AO", |compute_pass_builder| {
                    compute_pass_builder
                        .add_read_resource((gbuffer_ids[0], 0))
                        .add_read_resource((gbuffer_ids[1], 0))
                        .add_read_resource((gbuffer_ids[2], 0))
                        .add_read_resource((gbuffer_ids[3], 0))
                        .add_read_resource((depth_buffer_id, 0))
                        .add_write_resource((raw_ao_buffer_id, 0))
                        .execute(Box::new(|_| {
                            println!("AO pass execute");
                        }))
                })
                .add_compute_pass("BlurX", |compute_pass_builder| {
                    compute_pass_builder
                        .add_read_resource((raw_ao_buffer_id, 0))
                        .add_read_resource((depth_buffer_id, 0))
                        .add_write_resource((blur_buffer_id, 0))
                        .execute(Box::new(|_| {
                            println!("BlurX pass execute");
                        }))
                })
                .add_compute_pass("BlurY", |compute_pass_builder| {
                    compute_pass_builder
                        .add_read_resource((blur_buffer_id, 0))
                        .add_read_resource((depth_buffer_id, 0))
                        .add_write_resource((ao_buffer_id, 0))
                        .execute(Box::new(|_| {
                            println!("BlurY pass execute");
                        }))
                })
        })
    }

    #[allow(clippy::unused_self)]
    fn make_raw_ao_buffer_desc(&self, view: &RenderView) -> RenderTargetDesc {
        RenderTargetDesc {
            name: "RawAOBuffer".to_string(),
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8_UNORM,
        }
    }
}

impl UiPass {
    #[allow(clippy::unused_self)]
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        ui_buffer_id: RenderTargetId,
    ) -> RenderGraphBuilder {
        builder.add_graphics_pass("UI", |graphics_pass_builder| {
            graphics_pass_builder
                .add_render_target((ui_buffer_id, 0))
                .execute(Box::new(|_| {
                    println!("UiPass execute");
                }))
        })
    }
}
