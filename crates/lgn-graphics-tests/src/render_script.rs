use crate::render_passes::{
    AlphaBlendedLayerPass, DepthLayerPass, GpuCullingPass, LightingPass, OpaqueLayerPass,
    PostProcessPass, SSAOPass, UiPass,
};

use lgn_graphics_api::{
    BufferDef, BufferViewDef, BufferViewFlags, Extents3D, Format, GPUViewType, MemoryUsage,
    PlaneSlice, ResourceCreation, ResourceFlags, ResourceUsage, Texture, TextureDef, TextureTiling,
    TextureViewDef, ViewDimension, MAX_RENDER_TARGET_ATTACHMENTS,
};

///
///
/// `https://logins.github.io/graphics/2021/05/31/RenderGraphs.html`
/// `https://medium.com/embarkstudios/homegrown-rendering-with-rust-1e39068e56a7`
///
///
///
///

#[derive(Clone, Debug, PartialEq)]
pub struct RenderGraphTextureDef {
    // TextureDef
    pub extents: Extents3D,
    pub array_length: u32,
    pub total_mip_count: u32,
    pub format: Format,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderGraphTextureViewDef {
    // TextureViewDef
    pub view_dimension: ViewDimension,
    pub first_mip: u32,
    pub mip_count: u32,
    pub plane_slice: PlaneSlice,
    pub first_array_slice: u32,
    pub array_size: u32,
}

impl From<RenderGraphTextureDef> for TextureDef {
    fn from(item: RenderGraphTextureDef) -> Self {
        Self {
            name: "".to_string(), // TODO will be removed
            extents: item.extents,
            array_length: item.array_length,
            mip_count: item.total_mip_count,
            format: item.format,
            usage_flags: if item.format.has_depth() {
                // TODO: will depend on read / write and whether the format is depth/stencil
                ResourceUsage::AS_DEPTH_STENCIL | ResourceUsage::AS_SHADER_RESOURCE
            } else {
                ResourceUsage::AS_RENDER_TARGET | ResourceUsage::AS_SHADER_RESOURCE
            },
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        }
    }
}

impl From<RenderGraphTextureViewDef> for TextureViewDef {
    fn from(item: RenderGraphTextureViewDef) -> Self {
        Self {
            gpu_view_type: GPUViewType::ShaderResource, // TODO: will depend on read / write and whether the format is depth/stencil
            view_dimension: item.view_dimension,
            first_mip: item.first_mip,
            mip_count: item.mip_count,
            plane_slice: item.plane_slice,
            first_array_slice: item.first_array_slice,
            array_size: item.array_size,
        }
    }
}

impl From<TextureDef> for RenderGraphTextureDef {
    fn from(item: TextureDef) -> Self {
        Self {
            extents: item.extents,
            array_length: item.array_length,
            total_mip_count: item.mip_count,
            format: item.format,
        }
    }
}

impl From<TextureViewDef> for RenderGraphTextureViewDef {
    fn from(item: TextureViewDef) -> Self {
        Self {
            view_dimension: item.view_dimension,
            first_mip: item.first_mip,
            mip_count: item.mip_count,
            plane_slice: item.plane_slice,
            first_array_slice: item.first_array_slice,
            array_size: item.array_size,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderGraphBufferDef {
    // Buffer
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderGraphBufferViewDef {
    // BufferView
    pub byte_offset: u64,
    pub element_count: u64,
    pub element_size: u64,
    pub buffer_view_flags: BufferViewFlags,
}

impl From<RenderGraphBufferDef> for BufferDef {
    fn from(item: RenderGraphBufferDef) -> Self {
        Self {
            name: "".to_string(),
            size: item.size,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE, // TODO: will depend on read / write
            creation_flags: ResourceCreation::empty(), // TODO: do we want to give control on this?
        }
    }
}

impl From<RenderGraphBufferViewDef> for BufferViewDef {
    fn from(item: RenderGraphBufferViewDef) -> Self {
        Self {
            gpu_view_type: GPUViewType::ShaderResource, // TODO: will depend on read / write
            byte_offset: item.byte_offset,
            element_count: item.element_count,
            element_size: item.element_size,
            buffer_view_flags: item.buffer_view_flags,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RenderGraphResourceDef {
    Texture {
        definition: RenderGraphTextureDef,
    },
    #[allow(dead_code)]
    Buffer {
        definition: RenderGraphBufferDef,
    },
}

impl RenderGraphResourceDef {
    pub(crate) fn texture_def(&self) -> &RenderGraphTextureDef {
        match self {
            RenderGraphResourceDef::Texture { definition } => definition,
            RenderGraphResourceDef::Buffer { .. } => panic!("Type is not a texture def."),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn buffer_def(&self) -> &RenderGraphBufferDef {
        match self {
            RenderGraphResourceDef::Texture { .. } => panic!("Type is not a buffer def."),
            RenderGraphResourceDef::Buffer { definition } => definition,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn texture_def_mut(&mut self) -> &mut RenderGraphTextureDef {
        match self {
            RenderGraphResourceDef::Texture { ref mut definition } => definition,
            RenderGraphResourceDef::Buffer { .. } => panic!("Type is not a texture def."),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn buffer_def_mut(&mut self) -> &mut RenderGraphBufferDef {
        match self {
            RenderGraphResourceDef::Texture { .. } => panic!("Type is not a buffer def."),
            RenderGraphResourceDef::Buffer { ref mut definition } => definition,
        }
    }
}

pub type RenderGraphResourceId = u64;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RenderGraphViewDef {
    Texture {
        definition: RenderGraphTextureViewDef,
    },
    #[allow(dead_code)]
    Buffer {
        definition: RenderGraphBufferViewDef,
    },
}

impl RenderGraphViewDef {
    pub(crate) fn texture_view_def(&self) -> &RenderGraphTextureViewDef {
        match self {
            RenderGraphViewDef::Texture { definition } => definition,
            RenderGraphViewDef::Buffer { .. } => panic!("Type is not a texture def."),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn buffer_view_def(&self) -> &RenderGraphBufferViewDef {
        match self {
            RenderGraphViewDef::Texture { .. } => panic!("Type is not a buffer def."),
            RenderGraphViewDef::Buffer { definition } => definition,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn texture_view_def_mut(&mut self) -> &mut RenderGraphTextureViewDef {
        match self {
            RenderGraphViewDef::Texture { ref mut definition } => definition,
            RenderGraphViewDef::Buffer { .. } => panic!("Type is not a texture def."),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn buffer_view_def_mut(&mut self) -> &mut RenderGraphBufferViewDef {
        match self {
            RenderGraphViewDef::Texture { .. } => panic!("Type is not a buffer def."),
            RenderGraphViewDef::Buffer { ref mut definition } => definition,
        }
    }
}

pub type RenderGraphViewId = u64;

pub struct GfxError {
    pub msg: String,
}
pub type GfxResult<T> = Result<T, GfxError>;

pub struct RenderView {
    pub target: Texture,
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

pub struct RenderGraphExecuteContext<'a> {
    pub(crate) name: String,
    pub(crate) resources: &'a Vec<RenderGraphResourceDef>,
}
type RenderGraphExecuteFn = dyn Fn(&RenderGraphExecuteContext<'_>);

struct RGNode {
    name: String,
    read_resources: Vec<(RenderGraphResourceId, RenderGraphViewId)>,
    write_resources: Vec<(RenderGraphResourceId, RenderGraphViewId)>,
    render_targets: Vec<Option<(RenderGraphResourceId, RenderGraphViewId)>>,
    depth_stencil: Option<(RenderGraphResourceId, RenderGraphViewId)>,
    children: Vec<RGNode>,
    execute_fn: Option<Box<RenderGraphExecuteFn>>,
}

impl Default for RGNode {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            read_resources: vec![],
            write_resources: vec![],
            render_targets: vec![None; MAX_RENDER_TARGET_ATTACHMENTS],
            depth_stencil: None,
            children: vec![],
            execute_fn: None,
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
    pub fn print(
        &self,
        indent: usize,
        resources: &Vec<RenderGraphResourceDef>,
        resource_names: &Vec<String>,
        views: &Vec<RenderGraphViewDef>,
    ) -> String {
        let indent_str = make_indent_string(indent);
        let mut str = format!("{}*-{}\n", indent_str, self.name);

        let iter = self.render_targets.iter().flatten();
        let render_targets = iter.collect::<Vec<&(u64, u64)>>();
        if !render_targets.is_empty() {
            str += &format!("{}  | Render targets:\n", indent_str);

            for res in render_targets {
                str += &format!(
                    "{}  |   {} mip {}\n",
                    indent_str,
                    resource_names[res.0 as usize],
                    views[res.1 as usize].texture_view_def().first_mip,
                );
            }
        }

        if let Some(depth_stencil) = &self.depth_stencil {
            str += &format!("{}  | Depth stencil:\n", indent_str);
            str += &format!(
                "{}  |   {} mip {}\n",
                indent_str,
                resource_names[depth_stencil.0 as usize],
                views[depth_stencil.1 as usize].texture_view_def().first_mip,
            );
        }

        if !self.read_resources.is_empty() {
            str += &format!("{}  | Reads:\n", indent_str);
            for res in &self.read_resources {
                str += &format!(
                    "{}  |   {} mip {}\n",
                    indent_str,
                    resource_names[res.0 as usize],
                    views[res.1 as usize].texture_view_def().first_mip,
                );
            }
        }

        if !self.write_resources.is_empty() {
            str += &format!("{}  | Writes:\n", indent_str);
            for res in &self.write_resources {
                str += &format!(
                    "{}  |   {} mip {}\n",
                    indent_str,
                    resource_names[res.0 as usize],
                    views[res.1 as usize].texture_view_def().first_mip,
                );
            }
        }

        for child in &self.children {
            str += &child.print(indent + 2, resources, resource_names, views);
        }
        str
    }
}

pub struct RenderGraph {
    root: RGNode,
    resources: Vec<RenderGraphResourceDef>,
    resource_names: Vec<String>,
    views: Vec<RenderGraphViewDef>,
}

impl RenderGraph {
    pub(crate) fn builder() -> RenderGraphBuilder {
        RenderGraphBuilder::default()
    }

    pub fn execute(&self) {
        let execute_context = RenderGraphExecuteContext {
            name: "Blah".to_string(),
            resources: &self.resources,
        };

        if let Some(execute_fn) = &self.root.execute_fn {
            (execute_fn)(&execute_context);
        }
        self.execute_inner(&self.root, &execute_context);
    }

    fn execute_inner(&self, node: &RGNode, execute_context: &RenderGraphExecuteContext<'_>) {
        for child in &node.children {
            if let Some(execute_fn) = &child.execute_fn {
                (execute_fn)(execute_context);
            }
            self.execute_inner(child, execute_context);
        }
    }
}

impl std::fmt::Display for RenderGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let printed = self
            .root
            .print(0, &self.resources, &self.resource_names, &self.views);
        write!(f, "{}", printed)
    }
}

pub(crate) struct GraphicsPassBuilder {
    node: RGNode,
}

impl GraphicsPassBuilder {
    pub fn read(mut self, resource: RenderGraphResourceId, view: RenderGraphViewId) -> Self {
        self.node.read_resources.push((resource, view));
        self
    }

    pub fn read_if(self, resource: Option<RenderGraphResourceId>, view: RenderGraphViewId) -> Self {
        if let Some(resource) = resource {
            self.read(resource, view)
        } else {
            self
        }
    }

    #[allow(dead_code)]
    pub fn write(mut self, resource: RenderGraphResourceId, view: RenderGraphViewId) -> Self {
        self.node.write_resources.push((resource, view));
        self
    }

    pub fn render_target(
        mut self,
        slot: u32,
        resource: RenderGraphResourceId,
        view: RenderGraphViewId,
    ) -> Self {
        self.node.render_targets[slot as usize] = Some((resource, view));
        self
    }

    pub fn depth_stencil(
        mut self,
        resource: RenderGraphResourceId,
        view: RenderGraphViewId,
    ) -> Self {
        self.node.depth_stencil = Some((resource, view));
        self
    }

    pub fn execute<F>(mut self, f: F) -> Self
    where
        F: Fn(&RenderGraphExecuteContext<'_>) + 'static,
    {
        self.node.execute_fn = Some(Box::new(f));
        self
    }
}

pub(crate) struct ComputePassBuilder {
    node: RGNode,
}

impl ComputePassBuilder {
    pub fn read(mut self, resource: RenderGraphResourceId, view: RenderGraphViewId) -> Self {
        self.node.read_resources.push((resource, view));
        self
    }

    pub fn write(mut self, resource: RenderGraphResourceId, view: RenderGraphViewId) -> Self {
        self.node.write_resources.push((resource, view));
        self
    }

    pub fn execute<F>(mut self, f: F) -> Self
    where
        F: Fn(&RenderGraphExecuteContext<'_>) + 'static,
    {
        self.node.execute_fn = Some(Box::new(f));
        self
    }
}

#[derive(Default)]
pub(crate) struct RenderGraphBuilder {
    current_parent: Option<RGNode>,
    resources: Vec<RenderGraphResourceDef>,
    resource_names: Vec<String>,
    next_resource_id: RenderGraphResourceId,
    views: Vec<RenderGraphViewDef>,
    next_view_id: RenderGraphViewId,
    top_level_nodes: Vec<RGNode>,
}

impl RenderGraphBuilder {
    pub fn declare_render_target(
        &mut self,
        name: &str,
        resource: &RenderGraphResourceDef,
    ) -> RenderGraphResourceId {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        self.resources.push(resource.clone());
        self.resource_names.push(name.to_string());
        id
    }

    pub fn inject_render_target(
        &mut self,
        name: &str,
        texture_def: &TextureDef,
    ) -> RenderGraphResourceId {
        // TEMP
        self.declare_render_target(
            name,
            &RenderGraphResourceDef::Texture {
                definition: texture_def.clone().into(),
            },
        )
    }

    pub fn declare_view(&mut self, view: &RenderGraphViewDef) -> RenderGraphViewId {
        let id = self.next_view_id;
        self.next_view_id += 1;
        self.views.push(view.clone());
        id
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

    pub fn get_resource_def(&self, resource_id: RenderGraphResourceId) -> &RenderGraphResourceDef {
        self.resources.get(resource_id as usize).unwrap()
    }

    #[allow(dead_code)]
    pub fn get_view_def(&self, view_id: RenderGraphViewId) -> &RenderGraphViewDef {
        self.views.get(view_id as usize).unwrap()
    }

    pub fn build(mut self) -> RenderGraph {
        if let Some(current_parent) = self.current_parent.take() {
            self.top_level_nodes.push(current_parent);
        }

        let root = RGNode {
            name: "root".to_string(),
            read_resources: vec![],
            write_resources: vec![],
            render_targets: vec![],
            depth_stencil: None,
            children: self.top_level_nodes,
            execute_fn: None,
        };

        // TODO: reads and writes should bubble up from child nodes to parents
        // TODO: transitions from write to read should insert a barrier

        RenderGraph {
            root,
            resources: self.resources,
            resource_names: self.resource_names,
            views: self.views,
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
    pub prev_depth: Texture,
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

        if view.target.definition().extents.width == 0
            || view.target.definition().extents.height == 0
            || view.target.definition().extents.depth != 1
            || view.target.definition().array_length != 1
        {
            return Err(GfxError {
                msg: "View target is invalid".to_string(),
            });
        }

        let view_target_id =
            rendergraph_builder.inject_render_target("ViewTarget", view.target.definition());

        let depth_buffer_desc = self.make_depth_buffer_desc(view);
        let depth_buffer_id =
            rendergraph_builder.declare_render_target("DepthBuffer", &depth_buffer_desc);
        let single_mip_view_def = self.make_single_mip_view_def();
        let single_mip_view_id = rendergraph_builder.declare_view(&single_mip_view_def);

        let gbuffer_descs = self.make_gbuffer_descs(view);
        let gbuffer_ids = [
            rendergraph_builder.declare_render_target("GBuffer0", &gbuffer_descs[0]),
            rendergraph_builder.declare_render_target("GBuffer1", &gbuffer_descs[1]),
            rendergraph_builder.declare_render_target("GBuffer2", &gbuffer_descs[2]),
            rendergraph_builder.declare_render_target("GBuffer3", &gbuffer_descs[3]),
        ];

        let radiance_buffer_desc = self.make_radiance_buffer_desc(view);
        let radiance_buffer_id =
            rendergraph_builder.declare_render_target("RadianceBuffer", &radiance_buffer_desc);

        let ao_buffer_desc = self.make_ao_buffer_desc(view);
        let ao_buffer_id = rendergraph_builder.declare_render_target("AOBuffer", &ao_buffer_desc);

        rendergraph_builder = self.depth_layer_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            single_mip_view_id,
        );
        rendergraph_builder = self.gpu_culling_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            single_mip_view_id,
        );
        rendergraph_builder = self.opaque_layer_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            single_mip_view_id,
            gbuffer_ids,
            single_mip_view_id,
        );
        rendergraph_builder = self.ssao_pass.build_render_graph(
            rendergraph_builder,
            view,
            depth_buffer_id,
            single_mip_view_id,
            gbuffer_ids,
            single_mip_view_id,
            ao_buffer_id,
            single_mip_view_id,
        );
        rendergraph_builder = self.lighting_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            single_mip_view_id,
            gbuffer_ids,
            single_mip_view_id,
            ao_buffer_id,
            single_mip_view_id,
            radiance_buffer_id,
            single_mip_view_id,
        );
        rendergraph_builder = self.alphablended_layer_pass.build_render_graph(
            rendergraph_builder,
            depth_buffer_id,
            single_mip_view_id,
            radiance_buffer_id,
            single_mip_view_id,
        );

        if config.display_post_process() {
            rendergraph_builder = self.postprocess_pass.build_render_graph(
                rendergraph_builder,
                radiance_buffer_id,
                single_mip_view_id,
            );
        }

        let ui_buffer_id = if config.display_ui() {
            let ui_buffer_desc = self.make_ui_buffer_desc(view);
            let ui_buffer_id =
                rendergraph_builder.declare_render_target("UIBuffer", &ui_buffer_desc);
            rendergraph_builder = self.ui_pass.build_render_graph(
                rendergraph_builder,
                ui_buffer_id,
                single_mip_view_id,
            );
            Some(ui_buffer_id)
        } else {
            None
        };

        rendergraph_builder = self.combine_pass(
            rendergraph_builder,
            view_target_id,
            single_mip_view_id,
            radiance_buffer_id,
            single_mip_view_id,
            ui_buffer_id,
            single_mip_view_id,
        );

        Ok(rendergraph_builder.build())
    }

    #[allow(clippy::unused_self)]
    fn make_depth_buffer_desc(&self, view: &RenderView) -> RenderGraphResourceDef {
        RenderGraphResourceDef::Texture {
            definition: RenderGraphTextureDef {
                extents: view.target.definition().extents,
                array_length: 1,
                total_mip_count: 1,
                format: Format::D24_UNORM_S8_UINT,
            },
        }
    }

    #[allow(clippy::unused_self)]
    fn make_single_mip_view_def(&self) -> RenderGraphViewDef {
        RenderGraphViewDef::Texture {
            definition: RenderGraphTextureViewDef {
                view_dimension: ViewDimension::_2D,
                first_mip: 0,
                mip_count: 1,
                plane_slice: PlaneSlice::Depth,
                first_array_slice: 0,
                array_size: 1,
            },
        }
    }

    #[allow(clippy::unused_self)]
    fn make_gbuffer_descs(&self, view: &RenderView) -> Vec<RenderGraphResourceDef> {
        let mut texture_def = RenderGraphTextureDef {
            extents: view.target.definition().extents,
            array_length: 1,
            total_mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
        };

        // Just to simulate different formats
        let gbuffer0_def = texture_def.clone();
        texture_def.format = Format::R16G16_SNORM; // Normals
        let gbuffer1_def = texture_def.clone();
        texture_def.format = Format::R8G8B8A8_UNORM;
        let gbuffer2_def = texture_def.clone();
        texture_def.format = Format::R8G8B8A8_UNORM;
        let gbuffer3_def = texture_def;

        vec![
            RenderGraphResourceDef::Texture {
                definition: gbuffer0_def,
            },
            RenderGraphResourceDef::Texture {
                definition: gbuffer1_def,
            },
            RenderGraphResourceDef::Texture {
                definition: gbuffer2_def,
            },
            RenderGraphResourceDef::Texture {
                definition: gbuffer3_def,
            },
        ]
    }

    #[allow(clippy::unused_self)]
    fn make_ao_buffer_desc(&self, view: &RenderView) -> RenderGraphResourceDef {
        RenderGraphResourceDef::Texture {
            definition: RenderGraphTextureDef {
                extents: view.target.definition().extents,
                array_length: 1,
                total_mip_count: 1,
                format: Format::R8_UNORM,
            },
        }
    }

    #[allow(clippy::unused_self)]
    fn make_radiance_buffer_desc(&self, view: &RenderView) -> RenderGraphResourceDef {
        RenderGraphResourceDef::Texture {
            definition: RenderGraphTextureDef {
                extents: view.target.definition().extents,
                array_length: 1,
                total_mip_count: 1,
                format: Format::R16G16B16A16_SFLOAT,
            },
        }
    }

    #[allow(clippy::unused_self)]
    fn make_ui_buffer_desc(&self, view: &RenderView) -> RenderGraphResourceDef {
        RenderGraphResourceDef::Texture {
            definition: RenderGraphTextureDef {
                extents: view.target.definition().extents,
                array_length: 1,
                total_mip_count: 1,
                format: Format::R8G8B8A8_UNORM,
            },
        }
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::too_many_arguments)]
    fn combine_pass(
        &self,
        builder: RenderGraphBuilder,
        view_target_id: RenderGraphResourceId,
        view_view_id: RenderGraphViewId,
        radiance_buffer_id: RenderGraphResourceId,
        radiance_view_id: RenderGraphViewId,
        ui_buffer_id: Option<RenderGraphResourceId>,
        ui_view_id: RenderGraphViewId,
    ) -> RenderGraphBuilder {
        builder.add_graphics_pass("Combine", |graphics_pass_builder| {
            graphics_pass_builder
                .read(radiance_buffer_id, radiance_view_id)
                .read_if(ui_buffer_id, ui_view_id)
                .render_target(0, view_target_id, view_view_id)
                .execute(|_| {
                    println!("Combine pass execute");
                })
        })
    }
}
