use std::fmt::Debug;

use crate::render_passes::{
    AlphaBlendedLayerPass, DepthLayerPass, GpuCullingPass, LightingPass, OpaqueLayerPass,
    PostProcessPass, SSAOPass, UiPass,
};

///
///
/// `https://logins.github.io/graphics/2021/05/31/RenderGraphs.html`
/// `https://medium.com/embarkstudios/homegrown-rendering-with-rust-1e39068e56a7`
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

#[derive(Clone, Debug)]
pub struct RenderTargetDesc {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub array_size: u32,
    pub format: Format,
}

pub type RenderTargetId = u64;

#[derive(Clone, Debug)]
pub struct RenderTarget {
    pub id: RenderTargetId,
    pub desc: RenderTargetDesc,
}

pub struct GfxError {
    pub msg: String,
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

type ResourceId = u64;
type ViewId = u64;

//struct Resource {
//    id: ResourceId,
//}

pub struct RenderGraphExecuteContext {
    pub name: String,
}
type RenderGraphExecuteFn = Box<dyn Fn(&RenderGraphExecuteContext) + 'static>;

struct RGNode {
    name: String,
    reads: Vec<(ResourceId, ViewId)>,
    writes: Vec<(ResourceId, ViewId)>,
    render_targets: Vec<(ResourceId, ViewId)>,
    depth_stencil: (ResourceId, ViewId),
    children: Vec<RGNode>,
    execute_fn: RenderGraphExecuteFn,
}

impl Default for RGNode {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            reads: vec![],
            writes: vec![],
            render_targets: vec![],
            depth_stencil: (0, 0),
            children: vec![],
            execute_fn: Box::new(|_| {}),
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
    pub fn print(&self, indent: usize, render_targets: &Vec<RenderTarget>) -> String {
        let indent_str = make_indent_string(indent);
        let mut str = format!("{}*-{}\n", indent_str, self.name);

        if !self.render_targets.is_empty() {
            str += &format!("{}  | Render targets:\n", indent_str);
            for res in &self.render_targets {
                str += &format!(
                    "{}  |   {} view {}\n",
                    indent_str, render_targets[res.0 as usize].desc.name, res.1
                );
            }
        }

        if self.depth_stencil != (0, 0) {
            str += &format!("{}  | Depth stencil:\n", indent_str);
            str += &format!(
                "{}  |   {} view {}\n",
                indent_str,
                render_targets[self.depth_stencil.0 as usize].desc.name,
                self.depth_stencil.1
            );
        }

        if !self.reads.is_empty() {
            str += &format!("{}  | Reads:\n", indent_str);
            for res in &self.reads {
                str += &format!(
                    "{}  |   {} view {}\n",
                    indent_str, render_targets[res.0 as usize].desc.name, res.1
                );
            }
        }

        if !self.writes.is_empty() {
            str += &format!("{}  | Writes:\n", indent_str);
            for res in &self.writes {
                str += &format!(
                    "{}  |   {} view {}\n",
                    indent_str, render_targets[res.0 as usize].desc.name, res.1
                );
            }
        }

        for child in &self.children {
            str += &child.print(indent + 2, render_targets);
        }
        str
    }
}

pub struct RenderGraph {
    root: RGNode,
    render_targets: Vec<RenderTarget>,
}

impl RenderGraph {
    pub(crate) fn builder() -> RenderGraphBuilder {
        RenderGraphBuilder::default()
    }

    pub fn execute(&self, execute_context: &RenderGraphExecuteContext) {
        (self.root.execute_fn)(execute_context);
        self.execute_inner(&self.root, execute_context);
    }

    fn execute_inner(&self, node: &RGNode, execute_context: &RenderGraphExecuteContext) {
        for child in &node.children {
            (child.execute_fn)(execute_context);
            self.execute_inner(child, execute_context);
        }
    }
}

impl std::fmt::Display for RenderGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let printed = self.root.print(0, &self.render_targets);
        write!(f, "{}", printed)
    }
}

pub(crate) struct GraphicsPassBuilder {
    node: RGNode,
}

impl GraphicsPassBuilder {
    pub fn add_read_resource(mut self, resource: (ResourceId, ViewId)) -> Self {
        self.node.reads.push(resource);
        self
    }

    pub fn add_read_resource_if(self, resource: Option<(ResourceId, ViewId)>) -> Self {
        if let Some(resource) = resource {
            self.add_read_resource(resource)
        } else {
            self
        }
    }

    #[allow(dead_code)]
    pub fn add_write_resource(mut self, resource: (ResourceId, ViewId)) -> Self {
        self.node.writes.push(resource);
        self
    }

    pub fn add_render_target(mut self, resource: (ResourceId, ViewId)) -> Self {
        self.node.render_targets.push(resource);
        self
    }

    pub fn add_depth_stencil(mut self, resources: (ResourceId, ViewId)) -> Self {
        self.node.depth_stencil = resources;
        self
    }

    pub fn execute(mut self, f: RenderGraphExecuteFn) -> Self {
        self.node.execute_fn = f;
        self
    }
}

pub(crate) struct ComputePassBuilder {
    node: RGNode,
}

impl ComputePassBuilder {
    pub fn add_read_resource(mut self, resource: (ResourceId, ViewId)) -> Self {
        self.node.reads.push(resource);
        self
    }

    pub fn add_write_resource(mut self, resource: (ResourceId, ViewId)) -> Self {
        self.node.writes.push(resource);
        self
    }

    pub fn execute(mut self, f: RenderGraphExecuteFn) -> Self {
        self.node.execute_fn = f;
        self
    }
}

#[derive(Default)]
pub(crate) struct RenderGraphBuilder {
    current_parent: Option<RGNode>,
    render_targets: Vec<RenderTarget>,
    next_render_target_id: RenderTargetId,
    top_level_nodes: Vec<RGNode>,
}

impl RenderGraphBuilder {
    pub fn declare_render_target(&mut self, desc: &RenderTargetDesc) -> RenderTargetId {
        let id = self.next_render_target_id;
        self.next_render_target_id += 1;
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

    pub fn add_graphics_pass<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(GraphicsPassBuilder) -> GraphicsPassBuilder,
    {
        let current_node = RGNode {
            name: name.to_string(),
            ..RGNode::default()
        };

        let graphics_pass_builder = GraphicsPassBuilder { node: current_node };
        let graphics_pass_builder = f(graphics_pass_builder);

        let current_node = graphics_pass_builder.node;
        if let Some(current_parent) = &mut self.current_parent {
            current_parent.children.push(current_node);
        } else {
            self.top_level_nodes.push(current_node);
        }

        self
    }

    pub fn add_compute_pass<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(ComputePassBuilder) -> ComputePassBuilder,
    {
        let current_node = RGNode {
            name: name.to_string(),
            ..RGNode::default()
        };

        let compute_pass_builder = ComputePassBuilder { node: current_node };
        let compute_pass_builder = f(compute_pass_builder);

        let current_node = compute_pass_builder.node;
        if let Some(current_parent) = &mut self.current_parent {
            current_parent.children.push(current_node);
        } else {
            self.top_level_nodes.push(current_node);
        }

        self
    }

    pub fn add_scope<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let mut old_current_parent = self.current_parent.take();
        self.current_parent = Some(RGNode {
            name: name.to_string(),
            ..RGNode::default()
        });

        self = f(self);

        let current_parent = self.current_parent.unwrap(); // self.current_parent is always Some() because it's set above
        if let Some(mut old_current_parent) = old_current_parent.take() {
            old_current_parent.children.push(current_parent);
            self.current_parent = Some(old_current_parent);
        } else {
            self.top_level_nodes.push(current_parent);
            self.current_parent = None;
        }
        self
    }

    pub fn build(mut self) -> RenderGraph {
        if let Some(current_parent) = self.current_parent.take() {
            self.top_level_nodes.push(current_parent);
        }

        let root = RGNode {
            name: "root".to_string(),
            reads: vec![],
            writes: vec![],
            render_targets: vec![],
            depth_stencil: (0, 0),
            children: self.top_level_nodes,
            execute_fn: Box::new(|_| {}),
        };

        // TODO: reads and writes should bubble up from child nodes to parents
        // TODO: transitions from write to read should insert a barrier

        RenderGraph {
            root,
            render_targets: self.render_targets,
        }
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
    /// .
    ///
    /// # Examples
    ///
    /// ```
    /// use lgn_graphics_tests::render_script::RenderScript;
    ///
    /// let mut render_script = ;
    /// let result = render_script.build_render_graph(view, config);
    /// assert_eq!(result, );
    /// assert_eq!(render_script, );
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn build_render_graph(
        &mut self,
        view: &RenderView,
        config: &Config,
    ) -> GfxResult<RenderGraph> {
        let mut rendergraph_builder = RenderGraph::builder();

        if view.target.desc.width == 0
            || view.target.desc.height == 0
            || view.target.desc.depth != 1
            || view.target.desc.array_size != 1
        {
            return Err(GfxError {
                msg: "View target is invalid".to_string(),
            });
        }

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

    #[allow(clippy::unused_self)]
    fn make_depth_buffer_desc(&self, view: &RenderView) -> RenderTargetDesc {
        RenderTargetDesc {
            name: "DepthBuffer".to_string(),
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::D24_UNORM_S8_UINT,
        }
    }

    #[allow(clippy::unused_self)]
    fn make_gbuffer_descs(&self, view: &RenderView) -> Vec<RenderTargetDesc> {
        let gbuffer0_desc = RenderTargetDesc {
            name: "GBuffer0".to_string(),
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8G8B8A8_UNORM,
        };

        let gbuffer1_desc = RenderTargetDesc {
            name: "GBuffer1".to_string(),
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8G8B8A8_UNORM,
        };

        let gbuffer2_desc = RenderTargetDesc {
            name: "GBuffer2".to_string(),
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8G8B8A8_UNORM,
        };

        let gbuffer3_desc = RenderTargetDesc {
            name: "GBuffer3".to_string(),
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8G8B8A8_UNORM,
        };

        vec![gbuffer0_desc, gbuffer1_desc, gbuffer2_desc, gbuffer3_desc]
    }

    #[allow(clippy::unused_self)]
    fn make_ao_buffer_desc(&self, view: &RenderView) -> RenderTargetDesc {
        RenderTargetDesc {
            name: "AOBuffer".to_string(),
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8_UNORM,
        }
    }

    #[allow(clippy::unused_self)]
    fn make_radiance_buffer_desc(&self, view: &RenderView) -> RenderTargetDesc {
        RenderTargetDesc {
            name: "RadianceBuffer".to_string(),
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R16G16B16A16_SFLOAT,
        }
    }

    #[allow(clippy::unused_self)]
    fn make_ui_buffer_desc(&self, view: &RenderView) -> RenderTargetDesc {
        RenderTargetDesc {
            name: "UIBuffer".to_string(),
            width: view.target.desc.width,
            height: view.target.desc.height,
            depth: view.target.desc.depth,
            array_size: view.target.desc.array_size,
            format: Format::R8G8B8A8_UNORM,
        }
    }

    #[allow(clippy::unused_self)]
    fn combine_pass(
        &self,
        builder: RenderGraphBuilder,
        view_target_id: RenderTargetId,
        radiance_buffer_id: RenderTargetId,
        ui_buffer_id: Option<RenderTargetId>,
    ) -> RenderGraphBuilder {
        builder.add_graphics_pass("Combine", |graphics_pass_builder| {
            let ui_buffer_pair = ui_buffer_id.map(|ui_buffer_id| (ui_buffer_id, 0));

            graphics_pass_builder
                .add_read_resource((radiance_buffer_id, 0))
                .add_read_resource_if(ui_buffer_pair)
                .add_render_target((view_target_id, 0))
                .execute(Box::new(|_| {
                    println!("Combine pass execute");
                }))
        })
    }
}
