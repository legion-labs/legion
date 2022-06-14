use parking_lot::RwLock;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use lgn_core::Handle;
use lgn_graphics_api::{
    BarrierQueueTransition, Buffer, BufferBarrier, BufferCreateFlags, BufferDef, BufferView,
    BufferViewDef, BufferViewFlags, CmdCopyBufferToTextureParams, ColorClearValue,
    ColorRenderTargetBinding, CommandBuffer, DepthStencilClearValue,
    DepthStencilRenderTargetBinding, DeviceContext, Extents3D, Format, GPUViewType, LoadOp,
    MemoryUsage, PlaneSlice, ResourceFlags, ResourceState, ResourceUsage, StoreOp, Texture,
    TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef, ViewDimension,
    MAX_RENDER_TARGET_ATTACHMENTS,
};
use lgn_tracing::span_scope;

use crate::core::render_graph::RenderGraphBuilder;
use crate::core::RenderViewport;
use crate::core::{RenderCamera, RenderListSet, RenderResources};
use crate::egui::Egui;

use crate::render_pass::PickingRenderPass;
use crate::resources::{PipelineManager, ReadbackBuffer};
use crate::RenderContext;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RenderGraphTextureDef {
    // TextureDef
    pub extents: Extents3D,
    pub array_length: u32,
    pub mip_count: u32,
    pub format: Format,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RenderGraphTextureViewDef {
    // TextureViewDef
    pub resource_id: RenderGraphResourceId,
    pub gpu_view_type: GPUViewType,
    pub view_dimension: ViewDimension,
    pub first_mip: u32,
    pub mip_count: u32,
    pub plane_slice: PlaneSlice,
    pub first_array_slice: u32,
    pub array_size: u32,
    pub read_only: bool,
    pub copy: bool,
}

impl From<RenderGraphTextureDef> for TextureDef {
    fn from(item: RenderGraphTextureDef) -> Self {
        Self {
            extents: item.extents,
            array_length: item.array_length,
            mip_count: item.mip_count,
            format: item.format,
            usage_flags: if item.format.has_depth() {
                ResourceUsage::AS_DEPTH_STENCIL
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_TRANSFERABLE
            } else {
                ResourceUsage::AS_RENDER_TARGET
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_UNORDERED_ACCESS
                    | ResourceUsage::AS_TRANSFERABLE
            },
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        }
    }
}

impl From<RenderGraphTextureViewDef> for TextureViewDef {
    fn from(item: RenderGraphTextureViewDef) -> Self {
        Self {
            gpu_view_type: item.gpu_view_type,
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
            mip_count: item.mip_count,
            format: item.format,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RenderGraphBufferDef {
    // Buffer
    pub element_size: u64,
    pub element_count: u64,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RenderGraphBufferViewDef {
    // BufferView
    pub resource_id: RenderGraphResourceId,
    pub gpu_view_type: GPUViewType,
    pub byte_offset: u64,
    pub element_count: u64,
    pub element_size: u64,
    pub buffer_view_flags: BufferViewFlags,
    pub copy: bool,
    pub indirect: bool,
}

impl From<RenderGraphBufferDef> for BufferDef {
    fn from(item: RenderGraphBufferDef) -> Self {
        Self {
            size: item.element_size * item.element_count,
            usage_flags: ResourceUsage::AS_INDIRECT_BUFFER
                | ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_UNORDERED_ACCESS
                | ResourceUsage::AS_TRANSFERABLE,
            create_flags: BufferCreateFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            always_mapped: false,
        }
    }
}

impl From<RenderGraphBufferViewDef> for BufferViewDef {
    fn from(item: RenderGraphBufferViewDef) -> Self {
        Self {
            gpu_view_type: item.gpu_view_type,
            byte_offset: item.byte_offset,
            element_count: item.element_count,
            element_size: item.element_size,
            buffer_view_flags: item.buffer_view_flags,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum RenderGraphResourceDef {
    Texture(RenderGraphTextureDef),
    Buffer(RenderGraphBufferDef),
}

impl RenderGraphResourceDef {
    #[allow(non_snake_case)]
    pub fn new_texture(
        width: u32,
        height: u32,
        depth: u32,
        array_length: u32,
        mip_count: u32,
        format: Format,
    ) -> Self {
        Self::Texture(RenderGraphTextureDef {
            extents: Extents3D {
                width,
                height,
                depth,
            },
            array_length,
            mip_count,
            format,
        })
    }

    pub fn new_buffer(element_size: u64, element_count: u64) -> Self {
        Self::Buffer(RenderGraphBufferDef {
            element_size,
            element_count,
        })
    }
}

impl<'a> TryFrom<&'a RenderGraphResourceDef> for &'a RenderGraphTextureDef {
    type Error = &'static str;

    fn try_from(value: &'a RenderGraphResourceDef) -> Result<Self, Self::Error> {
        match &value {
            RenderGraphResourceDef::Texture(texture_def) => Ok(texture_def),
            RenderGraphResourceDef::Buffer(_) => Err("Conversion of RenderGraphResourceDef to RenderGraphTextureDef failed because def contains a BufferDef."),
        }
    }
}

impl<'a> TryFrom<&'a RenderGraphResourceDef> for &'a RenderGraphBufferDef {
    type Error = &'static str;

    fn try_from(value: &'a RenderGraphResourceDef) -> Result<Self, Self::Error> {
        match &value {
            RenderGraphResourceDef::Texture(_) => Err("Conversion of RenderGraphResourceDef to RenderGraphBufferDef failed because def contains a TextureDef."),
            RenderGraphResourceDef::Buffer(buffer_def) => Ok(buffer_def),
        }
    }
}

pub type RenderGraphResourceId = u32;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum RenderGraphViewDef {
    Texture(RenderGraphTextureViewDef),
    Buffer(RenderGraphBufferViewDef),
}

impl RenderGraphViewDef {
    pub fn new_depth_texture_view(
        resource_id: RenderGraphResourceId,
        first_mip: u32,
        gpu_view_type: GPUViewType,
        read_only: bool,
    ) -> Self {
        Self::Texture(RenderGraphTextureViewDef {
            resource_id,
            gpu_view_type,
            view_dimension: ViewDimension::_2D,
            first_mip,
            mip_count: 1,
            plane_slice: PlaneSlice::Depth,
            first_array_slice: 0,
            array_size: 1,
            read_only,
            copy: false,
        })
    }

    pub fn new_texture_view_with_mips(
        resource_id: RenderGraphResourceId,
        first_mip: u32,
        mip_count: u32,
        gpu_view_type: GPUViewType,
        copy: bool,
    ) -> Self {
        Self::Texture(RenderGraphTextureViewDef {
            resource_id,
            gpu_view_type,
            view_dimension: ViewDimension::_2D,
            first_mip,
            mip_count,
            plane_slice: PlaneSlice::Default,
            first_array_slice: 0,
            array_size: 1,
            read_only: false,
            copy,
        })
    }

    pub fn new_uav_buffer_view(
        resource_id: RenderGraphResourceId,
        def: &RenderGraphResourceDef,
    ) -> Self {
        let def: &RenderGraphBufferDef = def.try_into().unwrap();
        Self::Buffer(RenderGraphBufferViewDef {
            resource_id,
            gpu_view_type: GPUViewType::UnorderedAccess,
            byte_offset: 0,
            element_count: def.element_count,
            element_size: def.element_size,
            buffer_view_flags: BufferViewFlags::empty(),
            copy: false,
            indirect: false,
        })
    }

    pub fn new_srv_buffer_view(
        resource_id: RenderGraphResourceId,
        def: &RenderGraphResourceDef,
    ) -> Self {
        let def: &RenderGraphBufferDef = def.try_into().unwrap();
        Self::Buffer(RenderGraphBufferViewDef {
            resource_id,
            gpu_view_type: GPUViewType::ShaderResource,
            byte_offset: 0,
            element_count: def.element_count,
            element_size: def.element_size,
            buffer_view_flags: BufferViewFlags::empty(),
            copy: false,
            indirect: false,
        })
    }

    pub fn new_indirect_buffer_view(
        resource_id: RenderGraphResourceId,
        def: &RenderGraphResourceDef,
    ) -> Self {
        let def: &RenderGraphBufferDef = def.try_into().unwrap();
        Self::Buffer(RenderGraphBufferViewDef {
            resource_id,
            gpu_view_type: GPUViewType::ShaderResource,
            byte_offset: 0,
            element_count: def.element_count,
            element_size: def.element_size,
            buffer_view_flags: BufferViewFlags::empty(),
            copy: false,
            indirect: true,
        })
    }

    pub fn new_copy_dst_buffer_view(
        resource_id: RenderGraphResourceId,
        def: &RenderGraphResourceDef,
    ) -> Self {
        let def: &RenderGraphBufferDef = def.try_into().unwrap();
        Self::Buffer(RenderGraphBufferViewDef {
            resource_id,
            gpu_view_type: GPUViewType::UnorderedAccess,
            byte_offset: 0,
            element_count: def.element_count,
            element_size: def.element_size,
            buffer_view_flags: BufferViewFlags::empty(),
            copy: true,
            indirect: false,
        })
    }
}

impl<'a> TryFrom<&'a RenderGraphViewDef> for &'a RenderGraphTextureViewDef {
    type Error = &'static str;

    fn try_from(value: &'a RenderGraphViewDef) -> Result<Self, Self::Error> {
        match &value {
            RenderGraphViewDef::Texture(texture_view_def) => Ok(texture_view_def),
            RenderGraphViewDef::Buffer(_) => Err("Conversion of RenderGraphViewDef to RenderGraphTextureViewDef failed because def contains a BufferViewDef."),
        }
    }
}

impl<'a> TryFrom<&'a RenderGraphViewDef> for &'a RenderGraphBufferViewDef {
    type Error = &'static str;

    fn try_from(value: &'a RenderGraphViewDef) -> Result<Self, Self::Error> {
        match &value {
            RenderGraphViewDef::Texture(_) => Err("Conversion of RenderGraphViewDef to RenderGraphBufferViewDef failed because def contains a TextureViewDef."),
            RenderGraphViewDef::Buffer(buffer_view_def) => Ok(buffer_view_def),
        }
    }
}

impl<'a> TryFrom<&'a mut RenderGraphViewDef> for &'a mut RenderGraphTextureViewDef {
    type Error = &'static str;

    fn try_from(value: &'a mut RenderGraphViewDef) -> Result<Self, Self::Error> {
        match value {
            RenderGraphViewDef::Texture(texture_view_def) => Ok(texture_view_def),
            RenderGraphViewDef::Buffer(_) => Err("Conversion of RenderGraphViewDef to RenderGraphTextureViewDef failed because def contains a BufferViewDef."),
        }
    }
}

impl<'a> TryFrom<&'a mut RenderGraphViewDef> for &'a mut RenderGraphBufferViewDef {
    type Error = &'static str;

    fn try_from(value: &'a mut RenderGraphViewDef) -> Result<Self, Self::Error> {
        match value {
            RenderGraphViewDef::Texture(_) => Err("Conversion of RenderGraphViewDef to RenderGraphBufferViewDef failed because def contains a TextureViewDef."),
            RenderGraphViewDef::Buffer(buffer_view_def) => Ok(buffer_view_def),
        }
    }
}

impl RenderGraphViewDef {
    fn get_resource_id(&self) -> RenderGraphResourceId {
        match self {
            RenderGraphViewDef::Texture(texture_view_def) => texture_view_def.resource_id,
            RenderGraphViewDef::Buffer(buffer_view_def) => buffer_view_def.resource_id,
        }
    }
}

pub type RenderGraphViewId = u32;

#[derive(Clone)]
pub enum RenderGraphLoadState {
    DontCare,
    Load,
    ClearColor(ColorClearValue),
    ClearDepthStencil(DepthStencilClearValue),
    ClearValue(u32),
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum RenderGraphStoreState {
    DontCare,
    Store,
}

#[derive(Clone)]
pub struct ResourceData {
    pub key: RenderGraphViewId,
    pub load_state: RenderGraphLoadState,
}

pub type RenderGraphExecuteFn =
    dyn Fn(&RenderGraphContext, &mut RenderGraphExecuteContext<'_, '_>, &mut CommandBuffer);

pub struct RGNode {
    pub name: String,
    pub read_resources: Vec<ResourceData>,
    pub write_resources: Vec<ResourceData>,
    pub render_targets: Vec<Option<ResourceData>>,
    pub depth_stencil: Option<ResourceData>,
    pub children: Vec<RGNode>,
    pub execute_fn: Option<Box<RenderGraphExecuteFn>>,
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
        let render_targets = iter.collect::<Vec<&ResourceData>>();
        if !render_targets.is_empty() {
            str += &format!("{}  | Render targets:\n", indent_str);

            for res in render_targets {
                let view_def: &RenderGraphTextureViewDef =
                    (&views[res.key as usize]).try_into().unwrap();
                let resource_id = view_def.resource_id;
                str += &format!(
                    "{}  |   {} mip {}\n",
                    indent_str, resource_names[resource_id as usize], view_def.first_mip,
                );
            }
        }

        if let Some(depth_stencil) = &self.depth_stencil {
            let view_def: &RenderGraphTextureViewDef =
                (&views[depth_stencil.key as usize]).try_into().unwrap();
            let resource_id = view_def.resource_id;

            str += &format!("{}  | Depth stencil:\n", indent_str);
            str += &format!(
                "{}  |   {} mip {}\n",
                indent_str, resource_names[resource_id as usize], view_def.first_mip,
            );
        }

        if !self.read_resources.is_empty() {
            str += &format!("{}  | Reads:\n", indent_str);
            for res in &self.read_resources {
                match &views[res.key as usize] {
                    RenderGraphViewDef::Texture(texture_view_def) => {
                        let resource_id = texture_view_def.resource_id;
                        str += &format!(
                            "{}  |   {} mip {}\n",
                            indent_str,
                            resource_names[resource_id as usize],
                            texture_view_def.first_mip,
                        );
                    }
                    RenderGraphViewDef::Buffer(buffer_view_def) => {
                        let resource_id = buffer_view_def.resource_id;
                        str += &format!(
                            "{}  |   {}\n",
                            indent_str, resource_names[resource_id as usize],
                        );
                    }
                }
            }
        }

        if !self.write_resources.is_empty() {
            str += &format!("{}  | Writes:\n", indent_str);
            for res in &self.write_resources {
                match &views[res.key as usize] {
                    RenderGraphViewDef::Texture(texture_view_def) => {
                        let resource_id = texture_view_def.resource_id;
                        str += &format!(
                            "{}  |   {} mip {}\n",
                            indent_str,
                            resource_names[resource_id as usize],
                            texture_view_def.first_mip,
                        );
                    }
                    RenderGraphViewDef::Buffer(buffer_view_def) => {
                        let resource_id = buffer_view_def.resource_id;
                        str += &format!(
                            "{}  |   {}\n",
                            indent_str, resource_names[resource_id as usize],
                        );
                    }
                }
            }
        }

        for child in &self.children {
            str += &child.print(indent + 2, resources, resource_names, views);
        }
        str
    }
}

#[derive(Clone, PartialEq)]
pub enum RenderGraphResource {
    Texture(Texture),
    Buffer(Buffer),
}

impl<'a> TryFrom<&'a RenderGraphResource> for &'a Texture {
    type Error = &'static str;

    fn try_from(value: &'a RenderGraphResource) -> Result<Self, Self::Error> {
        match value {
            RenderGraphResource::Texture(texture) => Ok(texture),
            RenderGraphResource::Buffer(_) => Err("Conversion of RenderGraphResource to Texture failed because resource contains a Buffer."),
        }
    }
}

impl<'a> TryFrom<&'a RenderGraphResource> for &'a Buffer {
    type Error = &'static str;

    fn try_from(value: &'a RenderGraphResource) -> Result<Self, Self::Error> {
        match value {
            RenderGraphResource::Texture(_) => Err("Conversion of RenderGraphResource to Buffer failed because resource contains a Texture."),
            RenderGraphResource::Buffer(buffer) => Ok(buffer),
        }
    }
}

#[derive(Clone)]
pub enum RenderGraphView {
    TextureView(TextureView),
    BufferView(BufferView),
}

impl<'a> TryFrom<&'a RenderGraphView> for &'a TextureView {
    type Error = &'static str;

    fn try_from(value: &'a RenderGraphView) -> Result<Self, Self::Error> {
        match value {
            RenderGraphView::TextureView(texture_view) => Ok(texture_view),
            RenderGraphView::BufferView(_) => Err("Conversion of RenderGraphView to TextureView failed because view contains a BufferView."),
        }
    }
}

impl<'a> TryFrom<&'a mut RenderGraphView> for &'a mut TextureView {
    type Error = &'static str;

    fn try_from(value: &'a mut RenderGraphView) -> Result<Self, Self::Error> {
        match value {
            RenderGraphView::TextureView(texture_view) => Ok(texture_view),
            RenderGraphView::BufferView(_) => Err("Conversion of RenderGraphView to TextureView failed because view contains a BufferView."),
        }
    }
}

impl<'a> TryFrom<&'a RenderGraphView> for &'a BufferView {
    type Error = &'static str;

    fn try_from(value: &'a RenderGraphView) -> Result<Self, Self::Error> {
        match value {
            RenderGraphView::TextureView(_) => Err("Conversion of RenderGraphView to BufferView failed because view contains a TextureView."),
            RenderGraphView::BufferView(buffer_view) => Ok(buffer_view),
        }
    }
}

impl<'a> TryFrom<&'a mut RenderGraphView> for &'a mut BufferView {
    type Error = &'static str;

    fn try_from(value: &'a mut RenderGraphView) -> Result<Self, Self::Error> {
        match value {
            RenderGraphView::TextureView(_) => Err("Conversion of RenderGraphView to BufferView failed because view contains a TextureView."),
            RenderGraphView::BufferView(buffer_view) => Ok(buffer_view),
        }
    }
}

pub struct RenderGraphContext {
    resource_state: HashMap<(RenderGraphResourceId, u8), ResourceState>,
    created: Vec<RenderGraphResourceId>,
    lifetimes: Vec<(*const RGNode, *const RGNode)>, // indexed by RenderGraphResourceId
    resources: Vec<Option<RenderGraphResource>>,    // indexed by RenderGraphResourceId
    views: Vec<Option<RenderGraphView>>,            // indexed by RenderGraphViewId
}

impl RenderGraphContext {
    pub fn get_texture(&self, res_id: RenderGraphResourceId) -> &Texture {
        let texture = self.resources[res_id as usize].as_ref().unwrap();
        texture.try_into().unwrap()
    }

    pub fn get_buffer(&self, res_id: RenderGraphResourceId) -> &Buffer {
        let buffer = self.resources[res_id as usize].as_ref().unwrap();
        buffer.try_into().unwrap()
    }

    pub fn get_texture_view(&self, view_id: RenderGraphViewId) -> &TextureView {
        let view = self.views[view_id as usize].as_ref().unwrap();
        let view: &TextureView = view.try_into().unwrap();
        view
    }

    pub fn get_buffer_view(&self, view_id: RenderGraphViewId) -> &BufferView {
        let view = self.views[view_id as usize].as_ref().unwrap();
        let view: &BufferView = view.try_into().unwrap();
        view
    }
}

pub struct DebugStuff<'a> {
    pub picking_renderpass: &'a RwLock<PickingRenderPass>,
    pub render_viewport: &'a RenderViewport,
    pub render_camera: &'a RenderCamera,
    pub egui: &'a Egui,
}

pub struct RenderGraphExecuteContext<'a, 'frame> {
    // Managers and data used when rendering.
    pub render_list_set: &'a RenderListSet<'frame>,
    pub render_resources: &'a RenderResources,
    pub render_context: &'a mut RenderContext<'frame>,

    // Stuff needed only for the debug/picking/egui passes
    pub debug_stuff: &'a DebugStuff<'a>,

    // TODO(jsg): need a better way to pass data from one pass to another, like a blackboard
    // (but these buffers should be managed by the render graph anyways)
    pub count_readback: Handle<ReadbackBuffer>,
    pub picked_readback: Handle<ReadbackBuffer>,
}

pub struct RenderGraph {
    pub root_nodes: Vec<RGNode>,
    pub resource_defs: Vec<RenderGraphResourceDef>, // index is RenderGraphResourceId
    pub resource_names: Vec<String>,                // indexed by RenderGraphResourceId
    pub injected_resources: Vec<(RenderGraphResourceId, (RenderGraphResource, ResourceState))>,
    pub view_defs: Vec<RenderGraphViewDef>, // index is RenderGraphViewId
}

pub struct RenderGraphPersistentState {
    resources: RwLock<HashMap<String, (RenderGraphResourceDef, RenderGraphResource)>>,
    views: RwLock<HashMap<(RenderGraphViewDef, GPUViewType), RenderGraphView>>,
    injected_resources: RwLock<Vec<(RenderGraphResourceId, (RenderGraphResource, ResourceState))>>,
}

impl RenderGraphPersistentState {
    pub fn new() -> Self {
        Self {
            resources: RwLock::new(HashMap::new()),
            views: RwLock::new(HashMap::new()),
            injected_resources: RwLock::new(Vec::new()),
        }
    }

    pub fn get_resource(
        &self,
        name: &str,
        def: &RenderGraphResourceDef,
    ) -> Option<RenderGraphResource> {
        let mut need_destroy = false;

        {
            let resources = self.resources.read();
            if let Some(value) = resources.get(name) {
                if &value.0 == def {
                    return Some(value.1.clone());
                }

                need_destroy = true;
            }
        }

        if need_destroy {
            // Destroy resource and it will be recreated and re-added.
            // Also destroy all views related to this resource so they will also be recreated.
            let mut resources = self.resources.write();
            let mut views = self.views.write();
            let value = resources.get(name).unwrap();
            match &value.1 {
                RenderGraphResource::Texture(texture) => {
                    views.retain(|_, value| match value {
                        RenderGraphView::TextureView(texture_view) => {
                            texture_view.texture() != texture
                        }
                        RenderGraphView::BufferView(_) => true,
                    });
                }
                RenderGraphResource::Buffer(buffer) => {
                    views.retain(|_, value| match value {
                        RenderGraphView::BufferView(buffer_view) => buffer_view.buffer() != buffer,
                        RenderGraphView::TextureView(_) => true,
                    });
                }
            }
            resources.remove(name);
        }

        None
    }

    pub fn add_resource(
        &mut self,
        name: &str,
        def: &RenderGraphResourceDef,
        resource: &RenderGraphResource,
    ) {
        let mut resources = self.resources.write();
        resources.insert(name.to_string(), (def.clone(), resource.clone()));
    }

    pub fn get_view(
        &self,
        def: &RenderGraphViewDef,
        view_type: GPUViewType,
    ) -> Option<RenderGraphView> {
        {
            let views = self.views.read();
            if let Some(view) = views.get(&(def.clone(), view_type)) {
                return Some(view.clone());
            }
        }

        None
    }

    pub fn add_view(
        &mut self,
        def: &RenderGraphViewDef,
        view_type: GPUViewType,
        view: &RenderGraphView,
    ) {
        let mut views = self.views.write();
        views.insert((def.clone(), view_type), view.clone());
    }
}

struct ResourceBarrier {
    view_id: RenderGraphViewId,
    mip: u8,
    prev_state: ResourceState,
    next_state: ResourceState,
}

impl RenderGraph {
    pub fn builder<'a>(
        render_resources: &'a RenderResources,
        pipeline_manager: &'a mut PipelineManager,
        device_context: &'a DeviceContext,
    ) -> RenderGraphBuilder<'a> {
        RenderGraphBuilder::new(render_resources, pipeline_manager, device_context)
    }

    #[allow(clippy::unused_self)]
    fn get_previous_api_state(
        &self,
        context: &mut RenderGraphContext,
        res_mip_id: (RenderGraphResourceId, u8),
    ) -> ResourceState {
        *context
            .resource_state
            .entry(res_mip_id)
            .or_insert(ResourceState::UNDEFINED)
    }

    #[allow(clippy::unused_self)]
    fn get_texture_api_state(&self, texture_view_def: &RenderGraphTextureViewDef) -> ResourceState {
        match texture_view_def.gpu_view_type {
            GPUViewType::ShaderResource => {
                if texture_view_def.copy {
                    ResourceState::COPY_SRC
                } else {
                    ResourceState::SHADER_RESOURCE
                }
            }
            GPUViewType::UnorderedAccess => {
                if texture_view_def.copy {
                    ResourceState::COPY_DST
                } else {
                    ResourceState::UNORDERED_ACCESS
                }
            }
            GPUViewType::RenderTarget => ResourceState::RENDER_TARGET,
            GPUViewType::DepthStencil => {
                if texture_view_def.read_only {
                    ResourceState::DEPTH_READ
                } else {
                    ResourceState::DEPTH_WRITE
                }
            }
            GPUViewType::ConstantBuffer => panic!(),
        }
    }

    #[allow(clippy::unused_self)]
    fn get_buffer_api_state(&self, buffer_view_def: &RenderGraphBufferViewDef) -> ResourceState {
        match buffer_view_def.gpu_view_type {
            GPUViewType::ShaderResource => {
                if buffer_view_def.copy {
                    ResourceState::COPY_SRC
                } else if buffer_view_def.indirect {
                    ResourceState::INDIRECT_ARGUMENT
                } else {
                    ResourceState::SHADER_RESOURCE
                }
            }
            GPUViewType::UnorderedAccess => {
                if buffer_view_def.copy {
                    ResourceState::COPY_DST
                } else {
                    ResourceState::UNORDERED_ACCESS
                }
            }
            _ => panic!(),
        }
    }

    fn create_texture(
        &self,
        resource_id: RenderGraphResourceId,
        context: &mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_, '_>,
    ) {
        let res_idx = resource_id as usize;

        if !context.created.iter().any(|r| *r == resource_id) {
            if !self.injected_resources.iter().any(|r| r.0 == resource_id) {
                let name = &self.get_resource_name(resource_id);
                let original_texture_def = &self.resource_defs[res_idx];
                let texture_def: &RenderGraphTextureDef = original_texture_def.try_into().unwrap();
                let texture_def: TextureDef = texture_def.clone().into();

                let mut persistent_state = execute_context
                    .render_resources
                    .get_mut::<RenderGraphPersistentState>();

                if let Some(texture) = persistent_state.get_resource(name, original_texture_def) {
                    context.resources[res_idx] = Some(texture);
                } else {
                    //println!("  !! Create {} ", self.get_resource_name(resource_id));
                    let texture = execute_context
                        .render_context
                        .device_context
                        .create_texture(texture_def, self.get_resource_name(resource_id));
                    let texture = RenderGraphResource::Texture(texture);
                    persistent_state.add_resource(name, original_texture_def, &texture);
                    context.resources[res_idx] = Some(texture);
                }

                for mip in 0..texture_def.mip_count {
                    let res_mip_id = (res_idx as u32, mip as u8);
                    context
                        .resource_state
                        .insert(res_mip_id, ResourceState::UNDEFINED);
                }
            }

            context.created.push(resource_id);
        }
    }

    fn create_buffer(
        &self,
        resource_id: RenderGraphResourceId,
        context: &mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_, '_>,
    ) {
        let res_idx = resource_id as usize;

        if !context.created.iter().any(|r| *r == resource_id) {
            if !self.injected_resources.iter().any(|r| r.0 == resource_id) {
                let name = &self.get_resource_name(resource_id);
                let original_buffer_def = &self.resource_defs[res_idx];
                let buffer_def: &RenderGraphBufferDef = original_buffer_def.try_into().unwrap();
                let buffer_def: BufferDef = buffer_def.clone().into();

                let mut persistent_state = execute_context
                    .render_resources
                    .get_mut::<RenderGraphPersistentState>();

                if let Some(buffer) = persistent_state.get_resource(name, original_buffer_def) {
                    context.resources[res_idx] = Some(buffer);
                } else {
                    //println!("  !! Create {} ", self.get_resource_name(resource_id));
                    let buffer = execute_context
                        .render_context
                        .device_context
                        .create_buffer(buffer_def, self.get_resource_name(resource_id));
                    let buffer = RenderGraphResource::Buffer(buffer);
                    persistent_state.add_resource(name, original_buffer_def, &buffer);
                    context.resources[res_idx] = Some(buffer);
                }

                let res_mip_id = (res_idx as u32, 0);
                context
                    .resource_state
                    .insert(res_mip_id, ResourceState::UNDEFINED);
            }

            context.created.push(resource_id);
        }
    }

    #[allow(clippy::unused_self)]
    fn transition_texture<'a>(
        &self,
        res_mip_id: (RenderGraphResourceId, u8),
        texture: &'a Texture,
        prev_state: ResourceState,
        next_state: ResourceState,
        texture_barriers: &mut Vec<TextureBarrier<'a>>,
    ) {
        // println!(
        //     "  Transition texture {} mip {} from {:?} to {:?}",
        //     self.get_resource_name(res_mip_id.0),
        //     res_mip_id.1,
        //     prev_state,
        //     next_state,
        // );

        texture_barriers.push(TextureBarrier::state_transition_for_mip(
            texture,
            prev_state,
            next_state,
            Some(res_mip_id.1 as u8),
        ));
    }

    #[allow(clippy::unused_self)]
    fn transition_buffer<'a>(
        &self,
        _res_mip_id: (RenderGraphResourceId, u8),
        buffer: &'a Buffer,
        prev_state: ResourceState,
        next_state: ResourceState,
        buffer_barriers: &mut Vec<BufferBarrier<'a>>,
    ) {
        // println!(
        //     "  Transition buffer {} from {:?} to {:?}",
        //     self.get_resource_name(_res_mip_id.0),
        //     prev_state,
        //     next_state,
        // );

        buffer_barriers.push(BufferBarrier {
            buffer,
            src_state: prev_state,
            dst_state: next_state,
            queue_transition: BarrierQueueTransition::default(),
        });
    }

    fn gather_texture_transitions(
        &self,
        view_id: RenderGraphViewId,
        texture_view_def: &RenderGraphTextureViewDef,
        context: &mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_, '_>,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        let resource_id = texture_view_def.resource_id;
        let res_idx = texture_view_def.resource_id as usize;

        // Create if needed
        let mip_0_id = (res_idx as u32, 0);
        match context.resource_state.entry(mip_0_id) {
            Entry::Occupied(_) => {}
            Entry::Vacant(_) => {
                self.create_texture(resource_id, context, execute_context);
            }
        }

        assert!(
            context.resources[res_idx].is_some(),
            "Resource {} should have been created before being transitioned.",
            self.get_resource_name(resource_id)
        );

        // Gather transitions
        let first_mip = texture_view_def.first_mip;
        let mip_count = texture_view_def.mip_count;
        for mip in first_mip..first_mip + mip_count {
            let res_mip_id = (res_idx as u32, mip as u8);

            let prev_state = self.get_previous_api_state(context, res_mip_id);
            let next_state = self.get_texture_api_state(texture_view_def);

            if prev_state == next_state {
                // Nothing to do.
            } else {
                match context.resources[res_idx].as_ref().unwrap() {
                    RenderGraphResource::Texture(_) => {
                        barriers.push(ResourceBarrier {
                            view_id,
                            mip: mip as u8,
                            prev_state,
                            next_state,
                        });
                    }
                    RenderGraphResource::Buffer(_) => {
                        panic!("View was TextureView but Resource is Buffer?")
                    }
                }

                context.resource_state.insert(res_mip_id, next_state);
            }
        }
    }

    fn gather_buffer_transitions(
        &self,
        view_id: RenderGraphViewId,
        buffer_view_def: &RenderGraphBufferViewDef,
        context: &mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_, '_>,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        let resource_id = buffer_view_def.resource_id;
        let res_idx = buffer_view_def.resource_id as usize;

        // Create if needed
        let mip_0_id = (res_idx as u32, 0);
        match context.resource_state.entry(mip_0_id) {
            Entry::Occupied(_) => {}
            Entry::Vacant(_) => {
                self.create_buffer(resource_id, context, execute_context);
            }
        }

        assert!(
            context.resources[res_idx].is_some(),
            "Resource {} should have been created before being transitioned.",
            self.get_resource_name(resource_id)
        );

        // Gather transitions
        let res_mip_id = (res_idx as u32, 0);

        let prev_state = self.get_previous_api_state(context, res_mip_id);
        let next_state = self.get_buffer_api_state(buffer_view_def);

        if prev_state == next_state {
            // Nothing to do.
        } else {
            match context.resources[res_idx].as_ref().unwrap() {
                RenderGraphResource::Texture(_) => {
                    panic!("View was TextureView but Resource is Buffer?")
                }
                RenderGraphResource::Buffer(_) => {
                    barriers.push(ResourceBarrier {
                        view_id,
                        mip: 0,
                        prev_state,
                        next_state,
                    });
                }
            }

            context.resource_state.insert(res_mip_id, next_state);
        }
    }

    fn gather_resource_transitions<'a>(
        &self,
        view_id: RenderGraphViewId,
        context: &'a mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_, '_>,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        let view_idx = view_id as usize;

        let view_def = &self.view_defs[view_idx];
        match view_def {
            RenderGraphViewDef::Texture(texture_view_def) => {
                self.gather_texture_transitions(
                    view_id,
                    texture_view_def,
                    context,
                    execute_context,
                    barriers,
                );
            }
            RenderGraphViewDef::Buffer(buffer_view_def) => {
                self.gather_buffer_transitions(
                    view_id,
                    buffer_view_def,
                    context,
                    execute_context,
                    barriers,
                );
            }
        }
    }

    fn gather_read_resource_transitions(
        &self,
        context: &mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_, '_>,
        node: &RGNode,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        for read_res in &node.read_resources {
            self.gather_resource_transitions(read_res.key, context, execute_context, barriers);
        }
    }

    fn gather_write_resource_transitions<'a>(
        &self,
        context: &'a mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_, '_>,
        node: &RGNode,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        for write_res in &node.write_resources {
            self.gather_resource_transitions(write_res.key, context, execute_context, barriers);
        }
    }

    fn gather_rt_resource_transitions<'a>(
        &self,
        context: &'a mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_, '_>,
        node: &RGNode,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        for rt_res in node.render_targets.iter().flatten() {
            self.gather_resource_transitions(rt_res.key, context, execute_context, barriers);
        }
    }

    fn gather_depth_stencil_resource_transitions<'a>(
        &self,
        context: &'a mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_, '_>,
        node: &RGNode,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        if let Some(depth_stencil_res) = &node.depth_stencil {
            self.gather_resource_transitions(
                depth_stencil_res.key,
                context,
                execute_context,
                barriers,
            );
        }
    }

    fn do_resource_transitions(
        &self,
        context: &mut RenderGraphContext,
        execute_context: &mut RenderGraphExecuteContext<'_, '_>,
        node: &RGNode,
        cmd_buffer: &mut CommandBuffer,
    ) {
        // Gather barriers into a container.
        let mut barriers: Vec<ResourceBarrier> = Vec::with_capacity(32);

        self.gather_read_resource_transitions(context, execute_context, node, &mut barriers);

        self.gather_write_resource_transitions(context, execute_context, node, &mut barriers);

        self.gather_rt_resource_transitions(context, execute_context, node, &mut barriers);

        self.gather_depth_stencil_resource_transitions(
            context,
            execute_context,
            node,
            &mut barriers,
        );

        // Create the actual barriers
        let mut buffer_barriers: Vec<BufferBarrier<'_>> = Vec::with_capacity(32);
        let mut texture_barriers: Vec<TextureBarrier<'_>> = Vec::with_capacity(32);

        for barrier in &barriers {
            let view_idx = barrier.view_id as usize;
            match &self.view_defs[view_idx] {
                RenderGraphViewDef::Texture(texture_view_def) => {
                    let texture = context.get_texture(texture_view_def.resource_id);
                    let res_mip_id = (texture_view_def.resource_id, barrier.mip);
                    self.transition_texture(
                        res_mip_id,
                        texture,
                        barrier.prev_state,
                        barrier.next_state,
                        &mut texture_barriers,
                    );
                }
                RenderGraphViewDef::Buffer(buffer_view_def) => {
                    let buffer = context.get_buffer(buffer_view_def.resource_id);
                    let res_mip_id = (buffer_view_def.resource_id, 0);
                    self.transition_buffer(
                        res_mip_id,
                        buffer,
                        barrier.prev_state,
                        barrier.next_state,
                        &mut buffer_barriers,
                    );
                }
            }
        }

        // Execute the batch of barriers.
        cmd_buffer.cmd_resource_barrier(&buffer_barriers, &texture_barriers);
    }

    #[allow(clippy::unused_self)]
    fn need_begin_end_render_pass(&self, node: &RGNode) -> bool {
        node.render_targets.iter().flatten().next().is_some() || node.depth_stencil.is_some()
    }

    fn get_view_def(&self, view_id: RenderGraphViewId) -> &RenderGraphViewDef {
        &self.view_defs[view_id as usize]
    }

    fn get_texture_view_def(&self, view_id: RenderGraphViewId) -> &RenderGraphTextureViewDef {
        self.get_view_def(view_id).try_into().unwrap()
    }

    fn get_resource_name(&self, resource_id: RenderGraphResourceId) -> &String {
        &self.resource_names[resource_id as usize]
    }

    fn do_begin_render_pass(
        &self,
        context: &mut RenderGraphContext,
        node: &RGNode,
        cmd_buffer: &mut CommandBuffer,
    ) {
        if self.need_begin_end_render_pass(node) {
            let mut color_targets: Vec<ColorRenderTargetBinding<'_>> =
                Vec::with_capacity(node.render_targets.len());
            let mut depth_target: Option<DepthStencilRenderTargetBinding<'_>> = None;

            for resource_data in node.render_targets.iter().flatten() {
                let view_id = resource_data.key;
                let texture_view_def = self.get_texture_view_def(view_id);
                let resource_id = texture_view_def.resource_id;
                let texture_view = context.get_texture_view(view_id);

                let binding = ColorRenderTargetBinding {
                    texture_view,
                    load_op: match resource_data.load_state {
                        RenderGraphLoadState::DontCare => LoadOp::DontCare,
                        RenderGraphLoadState::Load => LoadOp::Load,
                        RenderGraphLoadState::ClearColor(_) => {
                            //println!("  !! Clear {} ", self.get_resource_name(resource_id));
                            LoadOp::Clear
                        }
                        RenderGraphLoadState::ClearDepthStencil(_) => {
                            panic!("Color render target binding {} cannot be cleared with a depth stencil clear value.", self.get_resource_name(resource_id));
                        }
                        RenderGraphLoadState::ClearValue(_) => {
                            panic!(
                                "Color render target binding {} cannot be cleared with a u32 clear value.", self.get_resource_name(resource_id)
                            );
                        }
                    },
                    store_op: StoreOp::Store,
                    clear_value: match resource_data.load_state {
                        RenderGraphLoadState::ClearColor(clear_value) => clear_value,
                        _ => ColorClearValue::default(),
                    },
                };
                color_targets.push(binding);
            }

            if let Some(resource_data) = &node.depth_stencil {
                let view_id = resource_data.key;
                let texture_view_def = self.get_texture_view_def(view_id);
                let resource_id = texture_view_def.resource_id;
                let texture_view = context.get_texture_view(view_id);

                depth_target = Some(DepthStencilRenderTargetBinding {
                    texture_view,
                    depth_load_op: match resource_data.load_state {
                        RenderGraphLoadState::DontCare => LoadOp::DontCare,
                        RenderGraphLoadState::Load => LoadOp::Load,
                        RenderGraphLoadState::ClearDepthStencil(_) => {
                            //println!("  !! Clear {} ", self.get_resource_name(resource_id));
                            LoadOp::Clear
                        }
                        RenderGraphLoadState::ClearColor(_) => {
                            panic!("Depth stencil render target binding {} cannot be cleared with a color clear value.", self.get_resource_name(resource_id));
                        }
                        RenderGraphLoadState::ClearValue(_) => {
                            panic!("Depth stencil render target binding {} cannot be cleared with a u32 clear value.", self.get_resource_name(resource_id));
                        }
                    },
                    depth_store_op: StoreOp::Store,
                    stencil_load_op: match resource_data.load_state {
                        RenderGraphLoadState::DontCare => LoadOp::DontCare,
                        RenderGraphLoadState::Load => LoadOp::Load,
                        RenderGraphLoadState::ClearDepthStencil(_) => LoadOp::Clear,
                        _ => {
                            panic!()
                        }
                    },
                    stencil_store_op: StoreOp::Store,
                    clear_value: match resource_data.load_state {
                        RenderGraphLoadState::ClearDepthStencil(clear_value) => clear_value,
                        _ => DepthStencilClearValue::default(),
                    },
                });
            }

            cmd_buffer.cmd_begin_render_pass(&color_targets, &depth_target);
        }
    }

    fn create_view(
        &self,
        context: &mut RenderGraphContext,
        execute_context: &mut RenderGraphExecuteContext<'_, '_>,
        resource_data: &ResourceData,
        view_type: GPUViewType,
    ) {
        let view_id = resource_data.key;
        let view_idx = resource_data.key as usize;

        if context.views.len() <= view_idx {
            context.views.resize(view_idx + 1, None);
        }

        if context.views[view_idx].is_none() {
            let view_def = self.get_view_def(view_id);

            let mut persistent_state = execute_context
                .render_resources
                .get_mut::<RenderGraphPersistentState>();

            if let Some(view) = persistent_state.get_view(view_def, view_type) {
                context.views[view_idx] = Some(view);
            } else {
                match view_def {
                    RenderGraphViewDef::Texture(texture_view_def) => {
                        let texture = context.get_texture(texture_view_def.resource_id);
                        let mut texture_view_def: TextureViewDef = texture_view_def.clone().into();
                        texture_view_def.gpu_view_type = view_type;
                        if view_type == GPUViewType::UnorderedAccess
                            || view_type == GPUViewType::RenderTarget
                            || view_type == GPUViewType::DepthStencil
                        {
                            assert_eq!(texture_view_def.mip_count, 1);
                        }
                        let texture_view_temp =
                            RenderGraphView::TextureView(texture.create_view(texture_view_def));
                        persistent_state.add_view(view_def, view_type, &texture_view_temp);
                        context.views[view_idx] = Some(texture_view_temp);
                    }
                    RenderGraphViewDef::Buffer(buffer_view_def) => {
                        let buffer = context.get_buffer(buffer_view_def.resource_id);
                        let mut buffer_view_def: BufferViewDef = buffer_view_def.clone().into();
                        buffer_view_def.gpu_view_type = view_type;
                        let buffer_view_temp =
                            RenderGraphView::BufferView(buffer.create_view(buffer_view_def));
                        persistent_state.add_view(view_def, view_type, &buffer_view_temp);
                        context.views[view_idx] = Some(buffer_view_temp);
                    }
                }
            }
        }
    }

    fn create_views(
        &self,
        context: &mut RenderGraphContext,
        execute_context: &mut RenderGraphExecuteContext<'_, '_>,
        node: &RGNode,
    ) {
        for resource_data in node.render_targets.iter().flatten() {
            self.create_view(
                context,
                execute_context,
                resource_data,
                GPUViewType::RenderTarget,
            );
        }

        if let Some(resource_data) = &node.depth_stencil {
            self.create_view(
                context,
                execute_context,
                resource_data,
                GPUViewType::DepthStencil,
            );
        }

        for resource_data in &node.read_resources {
            self.create_view(
                context,
                execute_context,
                resource_data,
                GPUViewType::ShaderResource,
            );
        }

        for resource_data in &node.write_resources {
            self.create_view(
                context,
                execute_context,
                resource_data,
                GPUViewType::UnorderedAccess,
            );
        }
    }

    fn upload_texture_data<T: Copy>(
        device_context: &DeviceContext,
        cmd_buffer: &mut CommandBuffer,
        texture: &Texture,
        data: &[T],
        outgoing_state: ResourceState,
    ) {
        //
        // TODO(vdbdd): this code should be moved (-> upload manager)
        // Motivations:
        // - Here the buffer is constantly reallocated
        // - Almost same code for buffer and texture
        // - Leverage the Copy queue
        //
        let staging_buffer = device_context.create_buffer(
            BufferDef::for_staging_buffer_data(data, ResourceUsage::empty()),
            "staging_buffer",
        );

        staging_buffer.copy_to_host_visible_buffer(data);

        cmd_buffer.cmd_resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                texture,
                ResourceState::UNDEFINED,
                ResourceState::COPY_DST,
            )],
        );

        cmd_buffer.cmd_copy_buffer_to_texture(
            &staging_buffer,
            texture,
            &CmdCopyBufferToTextureParams::default(),
        );

        cmd_buffer.cmd_resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                texture,
                ResourceState::COPY_DST,
                outgoing_state,
            )],
        );
    }

    fn clear_write_targets(
        &self,
        context: &mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_, '_>,
        node: &RGNode,
        cmd_buffer: &mut CommandBuffer,
    ) {
        for resource_data in &node.write_resources {
            let view_id = resource_data.key;
            let view_def = self.get_view_def(view_id);
            let resource_id = view_def.get_resource_id();

            match resource_data.load_state {
                RenderGraphLoadState::ClearValue(value) => {
                    //println!("  !! Clear {} ", self.get_resource_name(resource_id));
                    match view_def {
                        RenderGraphViewDef::Texture(_) => {
                            let texture = context.get_texture(resource_id);
                            let data = vec![value; texture.vk_alloc_size() as usize / 4];
                            Self::upload_texture_data(
                                execute_context.render_context.device_context,
                                cmd_buffer,
                                texture,
                                &data,
                                context.resource_state[&(resource_id, 0)],
                            );
                        }
                        RenderGraphViewDef::Buffer(_) => {
                            let buffer = context.get_buffer(resource_id);
                            cmd_buffer.cmd_fill_buffer(buffer, 0, buffer.definition().size, value);
                        }
                    }
                }
                RenderGraphLoadState::ClearColor(_) => {
                    panic!(
                        "Write target {} cannot be cleared with a color clear value.",
                        self.get_resource_name(resource_id)
                    );
                }
                RenderGraphLoadState::ClearDepthStencil(_) => {
                    panic!(
                        "Write target {} cannot be cleared with a depth stencil clear value.",
                        self.get_resource_name(resource_id)
                    );
                }
                _ => {}
            }
        }
    }

    fn begin_execute(
        &self,
        context: &mut RenderGraphContext,
        execute_context: &mut RenderGraphExecuteContext<'_, '_>,
        node: &RGNode,
        cmd_buffer: &mut CommandBuffer,
    ) {
        // Batch up and execute resource transitions.
        self.do_resource_transitions(context, execute_context, node, cmd_buffer);

        // Create the views we will need for the next steps.
        self.create_views(context, execute_context, node);

        // Do begin render pass which will also clear render targets and depth stencil.
        self.do_begin_render_pass(context, node, cmd_buffer);

        // Clear any write targets that need to.
        self.clear_write_targets(context, execute_context, node, cmd_buffer);
    }

    fn end_execute(
        &self,
        context: &RenderGraphContext,
        node: &RGNode,
        cmd_buffer: &mut CommandBuffer,
    ) {
        if self.need_begin_end_render_pass(node) {
            cmd_buffer.cmd_end_render_pass();
        }

        for (resource_idx, lifetime) in context.lifetimes.iter().enumerate() {
            if lifetime.1 == node
                && !self
                    .injected_resources
                    .iter()
                    .any(|r| r.0 == resource_idx as u32)
            {
                // TODO(jsg): Deallocate resource to be able to reuse it later in the graph execution
                // println!(
                //     "  !! Destroy {}",
                //     self.get_resource_name(resource_idx as RenderGraphResourceId)
                // );
            }
        }
    }

    pub fn compile(&self) -> RenderGraphContext {
        span_scope!("compile_render_graph");
        let mut context = RenderGraphContext {
            resource_state: HashMap::with_capacity(self.resource_defs.len()),
            created: vec![],
            lifetimes: Vec::with_capacity(self.resource_defs.len()),
            resources: vec![None; self.resource_defs.len()],
            views: vec![None; self.view_defs.len()],
        };

        // Add injected resources since they are already created (outside the graph)
        for injected_resource in &self.injected_resources {
            let res_id = injected_resource.0;
            let res_idx = res_id as usize;
            let resource = injected_resource.1 .0.clone();
            let initial_state = injected_resource.1 .1;

            match &resource {
                RenderGraphResource::Texture(texture) => {
                    for mip in 0..texture.definition().mip_count {
                        let res_mip_id = (res_id, mip as u8);
                        context.resource_state.insert(res_mip_id, initial_state);
                    }
                }
                RenderGraphResource::Buffer(_) => {
                    let res_mip_id = (res_id, 0);
                    context.resource_state.insert(res_mip_id, initial_state);
                }
            }

            context.resources[res_idx] = Some(resource);
        }

        for (id, res) in self.resource_defs.iter().enumerate() {
            context
                .lifetimes
                .push(self.find_lifetime_start_and_end(id as u32, res));
        }

        context
    }

    pub fn execute<'rt>(
        &self,
        context: &mut RenderGraphContext,
        render_list_set: &RenderListSet<'rt>,
        render_resources: &RenderResources,
        render_context: &mut RenderContext<'rt>,
        debug_stuff: &DebugStuff<'_>,
    ) {
        span_scope!("execute_render_graph");

        let mut execute_context = RenderGraphExecuteContext {
            render_list_set,
            render_resources,
            render_context,
            debug_stuff,
            count_readback: Handle::invalid(),
            picked_readback: Handle::invalid(),
        };

        {
            let mut persistent_state = execute_context
                .render_resources
                .get_mut::<RenderGraphPersistentState>();

            // Destroy the views of any injected resource that was modified (for example, resized) since last frame.
            {
                let prev_frame_injected_resources = persistent_state.injected_resources.read();
                for prev_frame_injected_resource in prev_frame_injected_resources.iter() {
                    let val = self
                        .injected_resources
                        .iter()
                        .find(|val| prev_frame_injected_resource.1 .0 == val.1 .0);

                    // If an injected resource from the previous frame was not found in this frame's injected resources, destroy its views.
                    if val.is_none() {
                        let mut views = persistent_state.views.write();
                        views.retain(|key, _| {
                            key.0.get_resource_id() != prev_frame_injected_resource.0
                        });
                    }
                }
            }

            // Keep the injected resources in the persistent state for next frame.
            persistent_state.injected_resources = RwLock::new(self.injected_resources.clone());
        }

        for child in &self.root_nodes {
            let mut cmd_buffer_handle = execute_context
                .render_context
                .transient_commandbuffer_allocator
                .acquire();
            let cmd_buffer = cmd_buffer_handle.as_mut();

            cmd_buffer.begin();

            self.execute_inner(context, &mut execute_context, child, cmd_buffer);

            cmd_buffer.end();

            execute_context
                .render_context
                .graphics_queue
                .queue_mut()
                .submit(&[cmd_buffer], &[], &[], None);

            execute_context
                .render_context
                .transient_commandbuffer_allocator
                .release(cmd_buffer_handle);
        }
    }

    fn execute_inner(
        &self,
        context: &mut RenderGraphContext,
        execute_context: &mut RenderGraphExecuteContext<'_, '_>,
        node: &RGNode,
        cmd_buffer: &mut CommandBuffer,
    ) {
        // TODO: #1993 change that to node.name asap
        span_scope!("execute_inner");

        cmd_buffer.with_label(&node.name, |cmd_buffer| {
            if let Some(execute_fn) = &node.execute_fn {
                self.begin_execute(context, execute_context, node, cmd_buffer);
                (execute_fn)(context, execute_context, cmd_buffer);
                self.end_execute(context, node, cmd_buffer);
            }

            for child in &node.children {
                self.execute_inner(context, execute_context, child, cmd_buffer);
            }
        });
    }

    fn find_lifetime_start_and_end(
        &self,
        id: u32,
        res: &RenderGraphResourceDef,
    ) -> (*const RGNode, *const RGNode) {
        let mut first_node: Option<&RGNode> = None;
        let mut last_node: Option<&RGNode> = None;

        for child in &self.root_nodes {
            self.find_lifetime_start_and_end_inner(id, res, child, &mut first_node, &mut last_node);
        }

        let _injected = self.injected_resources.iter().any(|r| r.0 == id);

        assert!(
            first_node.is_some() && last_node.is_some(),
            "Resource {} is never used in the render graph (as read, write, rt or ds)",
            self.get_resource_name(id)
        );

        // println!(
        //     "Resource {} first_node {} last_node {} {}",
        //     self.get_resource_name(id),
        //     first_node.unwrap().name,
        //     last_node.unwrap().name,
        //     if _injected { "(injected)" } else { "" },
        // );

        (first_node.unwrap(), last_node.unwrap())
    }

    fn find_lifetime_start_and_end_inner<'a>(
        &self,
        id: u32,
        res: &RenderGraphResourceDef,
        node: &'a RGNode,
        first_node: &mut Option<&'a RGNode>,
        last_node: &mut Option<&'a RGNode>,
    ) {
        let resource_used = node.read_resources.iter().any(|resource_data| {
            let view_def = &self.view_defs[resource_data.key as usize];
            let resource_id = view_def.get_resource_id();
            resource_id == id
        });
        let resource_used = resource_used
            || node.write_resources.iter().any(|resource_data| {
                let view_def = &self.view_defs[resource_data.key as usize];
                let resource_id = view_def.get_resource_id();
                resource_id == id
            });
        let resource_used = resource_used
            || node
                .render_targets
                .iter()
                .any(|resource_data| match resource_data {
                    Some(resource_data) => {
                        let view_def = &self.view_defs[resource_data.key as usize];
                        let resource_id = view_def.get_resource_id();
                        resource_id == id
                    }
                    _ => false,
                });
        let resource_used = resource_used
            || match &node.depth_stencil {
                Some(resource_data) => {
                    let view_def = &self.view_defs[resource_data.key as usize];
                    let resource_id = view_def.get_resource_id();
                    resource_id == id
                }
                _ => false,
            };

        if resource_used {
            if first_node.is_none() {
                *first_node = Some(node);
            }

            *last_node = Some(node);
        }

        for child in &node.children {
            self.find_lifetime_start_and_end_inner(id, res, child, first_node, last_node);
        }
    }
}

impl<'a> std::fmt::Display for RenderGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root_nodes.iter().fold(Ok(()), |result, child| {
            result.and_then(|_| {
                let printed = child.print(
                    0,
                    &self.resource_defs,
                    &self.resource_names,
                    &self.view_defs,
                );
                write!(f, "{}", printed)
            })
        })
    }
}
