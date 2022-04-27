use crate::render_script::{
    Format, RenderGraphBuilder, RenderTargetDesc, RenderTargetId, RenderView,
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
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderTargetId,
    ) -> RenderGraphBuilder {
        let builder = builder
            .add_pass("DepthLayer")
            .with_shader(1000)
            .reads(vec![])
            .writes(vec![depth_buffer_id]);
        builder
    }
}

impl OpaqueLayerPass {
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderTargetId,
        gbuffer_ids: [RenderTargetId; 4],
    ) -> RenderGraphBuilder {
        let mut writes = Vec::from(gbuffer_ids);
        writes.push(depth_buffer_id);
        let builder = builder
            .add_pass("OpaqueLayer")
            .with_shader(2000)
            .reads(vec![])
            .writes(writes);
        builder
    }
}

impl AlphaBlendedLayerPass {
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderTargetId,
        radiance_buffer_id: RenderTargetId,
    ) -> RenderGraphBuilder {
        let builder = builder
            .add_pass("AlphaBlendedLayer")
            .with_shader(3000)
            .reads(vec![depth_buffer_id])
            .writes(vec![radiance_buffer_id]);
        builder
    }
}

impl PostProcessPass {
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        radiance_buffer_id: RenderTargetId,
    ) -> RenderGraphBuilder {
        let builder = builder
            .add_pass("PostProcess")
            .with_shader(4000)
            .reads(vec![])
            .writes(vec![radiance_buffer_id]);
        builder
    }
}

impl LightingPass {
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        depth_buffer_id: RenderTargetId,
        gbuffer_ids: [RenderTargetId; 4],
        ao_buffer_id: RenderTargetId,
        radiance_buffer_id: RenderTargetId,
    ) -> RenderGraphBuilder {
        let mut reads = Vec::from(gbuffer_ids);
        reads.push(depth_buffer_id);
        reads.push(ao_buffer_id);
        let builder = builder
            .add_pass("Lighting")
            .with_shader(5000)
            .reads(reads)
            .writes(vec![radiance_buffer_id]);
        builder
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
        let raw_ao_buffer_desc = self.make_raw_ao_buffer_desc(view);
        let mut builder = builder;
        let raw_ao_buffer_id = builder.declare_render_target(&raw_ao_buffer_desc);

        let blur_buffer_id = builder.declare_render_target(&raw_ao_buffer_desc);

        let mut reads = Vec::from(gbuffer_ids);
        reads.push(depth_buffer_id);
        let builder = builder
            .add_pass("SSAO")
            .add_children()
            .add_pass("AO")
            .with_shader(8000)
            .reads(reads)
            .writes(vec![raw_ao_buffer_id])
            .add_pass("BlurX")
            .with_shader(8001)
            .reads(vec![raw_ao_buffer_id, depth_buffer_id])
            .writes(vec![blur_buffer_id])
            .add_pass("BlurY")
            .with_shader(8002)
            .reads(vec![blur_buffer_id, depth_buffer_id])
            .writes(vec![ao_buffer_id])
            .end_children();
        builder
    }

    fn make_raw_ao_buffer_desc(&self, view: &RenderView) -> RenderTargetDesc {
        RenderTargetDesc {
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8_UNORM,
        }
    }
}

impl UiPass {
    pub(crate) fn build_render_graph(
        &self,
        builder: RenderGraphBuilder,
        ui_buffer_id: RenderTargetId,
    ) -> RenderGraphBuilder {
        let builder = builder
            .add_pass("UI")
            .with_shader(6000)
            .reads(vec![])
            .writes(vec![ui_buffer_id]);
        builder
    }
}
