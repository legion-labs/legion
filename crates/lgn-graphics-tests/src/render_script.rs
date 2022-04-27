use crate::render_passes::*;

///
///
/// https://logins.github.io/graphics/2021/05/31/RenderGraphs.html
///
///
///
///
///

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Format {
    R8_UNORM,
    D24_UNORM_S8_UINT,
    R8G8B8A8_UNORM,
    R16G16B16A16_SFLOAT,
}

#[derive(Clone)]
pub struct RenderTargetDesc {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub array_size: u32,
    pub format: Format,
}

pub type RenderTargetId = u64;

#[derive(Clone)]
pub struct RenderTarget {
    pub id: RenderTargetId,
    pub desc: RenderTargetDesc,
}

pub enum GfxError {
    String(String),
}
pub type GfxResult<T> = Result<T, GfxError>;

pub struct RenderView {
    pub target: RenderTarget,
}

pub struct Config {
    display_post_process: bool,
    display_ui: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display_post_process: true,
            display_ui: true,
        }
    }
}

impl Config {
    pub fn display_post_process(&self) -> bool {
        self.display_post_process
    }

    pub fn display_ui(&self) -> bool {
        self.display_ui
    }
}

type ShaderId = u64;

type ResourceId = u64;

//struct Resource {
//    id: ResourceId,
//}

#[derive(Debug)]
struct RGNode {
    name: String,
    shader_id: ShaderId,
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
    children: Vec<RGNode>,
}

impl Default for RGNode {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            shader_id: 0,
            reads: vec![],
            writes: vec![],
            children: vec![],
        }
    }
}

fn make_indent_string(len: usize) -> String {
    let mut indent = "".to_string();
    for _i in 0..len {
        indent += " ";
    }
    indent
}

impl RGNode {
    pub fn print(&self, indent: usize) -> String {
        let indent_str = make_indent_string(indent);
        let mut str = format!("{}{} ShaderID {}\n", indent_str, self.name, self.shader_id);
        if !self.reads.is_empty() {
            str += &format!("{}  Reads:\n", indent_str);
            for res in &self.reads {
                str += &format!("{}    {}\n", indent_str, res);
            }
        }
        if !self.writes.is_empty() {
            str += &format!("{}  Writes:\n", indent_str);
            for res in &self.writes {
                str += &format!("{}    {}\n", indent_str, res);
            }
        }
        for child in &self.children {
            str += &child.print(indent + 2);
        }
        str
    }
}

#[derive(Debug)]
pub struct RenderGraph {
    root: RGNode,
}

impl RenderGraph {
    pub(crate) fn builder() -> RenderGraphBuilder {
        RenderGraphBuilder::default()
    }
    pub fn render(&self) {}
}

impl std::fmt::Display for RenderGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let printed = self.root.print(0);
        write!(f, "{}", printed)
    }
}

pub(crate) struct RenderGraphBuilder {
    current_node: Option<RGNode>,
    current_parent: Option<RGNode>,
    render_targets: Vec<RenderTarget>,
    next_rendertarget_id: RenderTargetId,
    top_level_nodes: Vec<RGNode>,
}

impl Default for RenderGraphBuilder {
    fn default() -> Self {
        Self {
            current_node: None,
            current_parent: None,
            render_targets: vec![],
            next_rendertarget_id: 0,
            top_level_nodes: vec![],
        }
    }
}

impl RenderGraphBuilder {
    pub fn declare_render_target(&mut self, desc: &RenderTargetDesc) -> RenderTargetId {
        let id = self.next_rendertarget_id;
        self.next_rendertarget_id = self.next_rendertarget_id + 1;
        let render_target = RenderTarget {
            id,
            desc: desc.clone(),
        };
        self.render_targets.push(render_target);
        id
    }

    pub fn inject_render_target(&mut self, desc: &RenderTargetDesc) -> RenderTargetId {
        // TEMP
        self.declare_render_target(desc)
    }

    pub fn add_pass(mut self, name: &str) -> Self {
        if let Some(current_node) = self.current_node.take() {
            if let Some(current_parent) = &mut self.current_parent {
                current_parent.children.push(current_node);
            } else {
                self.top_level_nodes.push(current_node);
            }
        }
        let mut current_node = RGNode::default();
        current_node.name = name.to_string();
        self.current_node = Some(current_node);
        self
    }

    pub fn add_children(mut self) -> Self {
        if let Some(current_node) = self.current_node.take() {
            self.current_parent = Some(current_node);
        } else {
            panic!("method should be chained with add_pass so we have a current_node to work on");
        }
        self
    }

    pub fn end_children(mut self) -> Self {
        if let Some(mut current_parent) = self.current_parent.take() {
            if let Some(current_node) = self.current_node.take() {
                current_parent.children.push(current_node);
            }
            self.current_node = Some(current_parent);
        } else {
            panic!("method should be called after add_children otherwise we don't have a current_parent")
        }
        self
    }

    pub fn with_shader(mut self, shader_id: ShaderId) -> Self {
        if let Some(current_node) = &mut self.current_node {
            current_node.shader_id = shader_id;
        } else {
            panic!("method should be chained with add_pass so we have a current_node to work on");
        }
        self
    }

    pub fn reads(mut self, mut resources: Vec<ResourceId>) -> Self {
        if let Some(current_node) = &mut self.current_node {
            current_node.reads.append(&mut resources);
        } else {
            panic!("method should be chained with add_pass so we have a current_node to work on");
        }
        self
    }

    pub fn writes(mut self, mut resources: Vec<ResourceId>) -> Self {
        if let Some(current_node) = &mut self.current_node {
            current_node.writes.append(&mut resources);
        } else {
            panic!("method should be chained with add_pass so we have a current_node to work on");
        }
        self
    }

    pub fn build(mut self) -> RenderGraph {
        if let Some(current_node) = self.current_node.take() {
            if let Some(current_parent) = &mut self.current_parent {
                current_parent.children.push(current_node);
            } else {
                self.top_level_nodes.push(current_node);
            }
        }
        let root = RGNode {
            name: "root".to_string(),
            shader_id: 0,
            reads: vec![],
            writes: vec![],
            children: self.top_level_nodes,
        };

        // TODO: reads and writes should bubble up from child nodes to parents
        // TODO: transitions from write to read should insert a barrier

        RenderGraph { root }
    }
}

pub struct RenderScript {
    // Passes
    pub gpu_culling_pass: GpuCullingPass,
    pub depth_layer_pass: DepthLayerPass,
    pub opaque_layer_pass: OpaqueLayerPass,
    pub ssao_pass: SSAOPass,
    pub alphablended_layer_pass: AlphaBlendedLayerPass,
    pub postprocess_pass: PostProcessPass,
    pub lighting_pass: LightingPass,
    pub ui_pass: UiPass,

    // Resources
    pub prev_depth: RenderTarget,
}

impl RenderScript {
    pub fn build_render_graph(
        &mut self,
        view: &RenderView,
        config: &Config,
    ) -> GfxResult<RenderGraph> {
        let mut rendergraph_builder = RenderGraph::builder();

        let view_target_id = rendergraph_builder.inject_render_target(&view.target.desc);

        let depth_buffer_desc = self.make_depth_buffer_desc(view);
        let depth_buffer_id = rendergraph_builder.declare_render_target(&depth_buffer_desc);

        let gbuffer_descs = self.make_gbuffer_descs(view);
        let gbuffer_ids = [
            rendergraph_builder.declare_render_target(&gbuffer_descs[0]),
            rendergraph_builder.declare_render_target(&gbuffer_descs[1]),
            rendergraph_builder.declare_render_target(&gbuffer_descs[2]),
            rendergraph_builder.declare_render_target(&gbuffer_descs[3]),
        ];

        let radiance_buffer_desc = self.make_radiance_buffer_desc(view);
        let radiance_buffer_id = rendergraph_builder.declare_render_target(&radiance_buffer_desc);

        let ao_buffer_desc = self.make_ao_buffer_desc(view);
        let ao_buffer_id = rendergraph_builder.declare_render_target(&ao_buffer_desc);

        rendergraph_builder = self
            .depth_layer_pass
            .build_render_graph(rendergraph_builder, depth_buffer_id);
        rendergraph_builder = self.opaque_layer_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            gbuffer_ids,
        );
        rendergraph_builder = self.ssao_pass.build_render_graph(
            rendergraph_builder,
            view,
            depth_buffer_id,
            gbuffer_ids,
            ao_buffer_id,
        );
        rendergraph_builder = self.lighting_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            gbuffer_ids,
            ao_buffer_id,
            radiance_buffer_id,
        );
        rendergraph_builder = self.alphablended_layer_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            radiance_buffer_id,
        );

        if config.display_post_process() {
            rendergraph_builder = self
                .postprocess_pass
                .build_render_graph(rendergraph_builder, radiance_buffer_id);
        }

        let ui_buffer_id = if config.display_ui() {
            let ui_buffer_desc = self.make_ui_buffer_desc(view);
            let ui_buffer_id = rendergraph_builder.declare_render_target(&ui_buffer_desc);
            rendergraph_builder = self
                .ui_pass
                .build_render_graph(rendergraph_builder, ui_buffer_id);
            Some(ui_buffer_id)
        } else {
            None
        };

        rendergraph_builder = self.combine_pass(
            rendergraph_builder,
            view_target_id,
            radiance_buffer_id,
            ui_buffer_id,
        );

        Ok(rendergraph_builder.build())
    }

    fn make_depth_buffer_desc(&self, view: &RenderView) -> RenderTargetDesc {
        RenderTargetDesc {
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::D24_UNORM_S8_UINT,
        }
    }

    fn make_gbuffer_descs(&self, view: &RenderView) -> Vec<RenderTargetDesc> {
        let gbuffer0_desc = RenderTargetDesc {
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8G8B8A8_UNORM,
        };

        let gbuffer1_desc = RenderTargetDesc {
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8G8B8A8_UNORM,
        };

        let gbuffer2_desc = RenderTargetDesc {
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8G8B8A8_UNORM,
        };

        let gbuffer3_desc = RenderTargetDesc {
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8G8B8A8_UNORM,
        };

        vec![gbuffer0_desc, gbuffer1_desc, gbuffer2_desc, gbuffer3_desc]
    }

    fn make_ao_buffer_desc(&self, view: &RenderView) -> RenderTargetDesc {
        RenderTargetDesc {
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8_UNORM,
        }
    }

    fn make_radiance_buffer_desc(&self, view: &RenderView) -> RenderTargetDesc {
        RenderTargetDesc {
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R16G16B16A16_SFLOAT,
        }
    }

    fn make_ui_buffer_desc(&self, view: &RenderView) -> RenderTargetDesc {
        RenderTargetDesc {
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8G8B8A8_UNORM,
        }
    }

    fn combine_pass(
        &self,
        builder: RenderGraphBuilder,
        view_target_id: RenderTargetId,
        radiance_buffer_id: RenderTargetId,
        ui_buffer_id: Option<RenderTargetId>,
    ) -> RenderGraphBuilder {
        let mut reads = vec![radiance_buffer_id];
        if let Some(ui_buffer_id) = ui_buffer_id {
            reads.push(ui_buffer_id);
        }
        let builder = builder
            .add_pass("Combine")
            .with_shader(7000)
            .reads(reads)
            .writes(vec![view_target_id]);
        builder
    }
}
