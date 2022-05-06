use std::collections::HashMap;

use lgn_graphics_api::{
    BarrierQueueTransition, Buffer, BufferBarrier, BufferDef, BufferViewDef, BufferViewFlags,
    ColorClearValue, CommandBuffer, CommandBufferDef, CommandPool, DepthStencilClearValue,
    DeviceContext, Extents3D, Format, GPUViewType, MemoryUsage, PlaneSlice, ResourceCreation,
    ResourceFlags, ResourceState, ResourceUsage, Texture, TextureBarrier, TextureDef,
    TextureTiling, TextureViewDef, ViewDimension, MAX_RENDER_TARGET_ATTACHMENTS,
};

use crate::render_graph_builder::RenderGraphBuilder;

#[derive(Clone, Debug)]
pub enum ColorOrDepthClearValue {
    Color(ColorClearValue),
    Depth(DepthStencilClearValue),
}

impl PartialEq for ColorOrDepthClearValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ColorOrDepthClearValue::Color(c) => match other {
                ColorOrDepthClearValue::Color(oc) => c.0 == oc.0,
                ColorOrDepthClearValue::Depth(_) => false,
            },
            ColorOrDepthClearValue::Depth(c) => match other {
                ColorOrDepthClearValue::Color(_) => false,
                ColorOrDepthClearValue::Depth(oc) => c.depth == oc.depth && c.stencil == oc.stencil,
            },
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct RenderGraphTextureDef {
    // TextureDef
    pub extents: Extents3D,
    pub array_length: u32,
    pub mip_count: u32,
    pub format: Format,
    pub clear: bool,
    pub clear_value: ColorOrDepthClearValue,
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
    pub read_only: bool,
}

impl From<RenderGraphTextureDef> for TextureDef {
    fn from(item: RenderGraphTextureDef) -> Self {
        Self {
            name: "".to_string(), // TODO will be removed
            extents: item.extents,
            array_length: item.array_length,
            mip_count: item.mip_count,
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
            mip_count: item.mip_count,
            format: item.format,
            clear: false,
            clear_value: ColorOrDepthClearValue::Color(ColorClearValue([0.0; 4])),
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
            read_only: false,
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
    Texture(RenderGraphTextureDef),
    #[allow(dead_code)]
    Buffer(RenderGraphBufferDef),
}

impl RenderGraphResourceDef {
    pub(crate) fn texture_def(&self) -> &RenderGraphTextureDef {
        match self {
            RenderGraphResourceDef::Texture(texture_def) => texture_def,
            RenderGraphResourceDef::Buffer(_) => panic!("Type is not a texture def."),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn buffer_def(&self) -> &RenderGraphBufferDef {
        match self {
            RenderGraphResourceDef::Texture(_) => panic!("Type is not a buffer def."),
            RenderGraphResourceDef::Buffer(buffer_def) => buffer_def,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn texture_def_mut(&mut self) -> &mut RenderGraphTextureDef {
        match self {
            RenderGraphResourceDef::Texture(texture_def) => texture_def,
            RenderGraphResourceDef::Buffer(_) => panic!("Type is not a texture def."),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn buffer_def_mut(&mut self) -> &mut RenderGraphBufferDef {
        match self {
            RenderGraphResourceDef::Texture(_) => panic!("Type is not a buffer def."),
            RenderGraphResourceDef::Buffer(buffer_def) => buffer_def,
        }
    }
}

pub type RenderGraphResourceId = u32;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RenderGraphViewDef {
    Texture(RenderGraphTextureViewDef),
    #[allow(dead_code)]
    Buffer(RenderGraphBufferViewDef),
}

impl RenderGraphViewDef {
    pub(crate) fn texture_view_def(&self) -> &RenderGraphTextureViewDef {
        match self {
            RenderGraphViewDef::Texture(texture_def) => texture_def,
            RenderGraphViewDef::Buffer(_) => panic!("Type is not a texture def."),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn buffer_view_def(&self) -> &RenderGraphBufferViewDef {
        match self {
            RenderGraphViewDef::Texture(_) => panic!("Type is not a buffer def."),
            RenderGraphViewDef::Buffer(buffer_def) => buffer_def,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn texture_view_def_mut(&mut self) -> &mut RenderGraphTextureViewDef {
        match self {
            RenderGraphViewDef::Texture(texture_def) => texture_def,
            RenderGraphViewDef::Buffer(_) => panic!("Type is not a texture def."),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn buffer_view_def_mut(&mut self) -> &mut RenderGraphBufferViewDef {
        match self {
            RenderGraphViewDef::Texture(_) => panic!("Type is not a buffer def."),
            RenderGraphViewDef::Buffer(buffer_def) => buffer_def,
        }
    }
}

pub type RenderGraphViewId = u32;

type RenderGraphExecuteFn = dyn Fn(&RenderGraphExecuteContext<'_>);

pub(crate) struct RGNode {
    pub(crate) name: String,
    pub(crate) read_resources: Vec<(RenderGraphResourceId, RenderGraphViewId)>,
    pub(crate) write_resources: Vec<(RenderGraphResourceId, RenderGraphViewId)>,
    pub(crate) render_targets: Vec<Option<(RenderGraphResourceId, RenderGraphViewId)>>,
    pub(crate) depth_stencil: Option<(RenderGraphResourceId, RenderGraphViewId)>,
    pub(crate) children: Vec<RGNode>,
    pub(crate) execute_fn: Option<Box<RenderGraphExecuteFn>>,
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
        let render_targets = iter.collect::<Vec<&(RenderGraphResourceId, RenderGraphViewId)>>();
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderGraphResourceState {
    Unknown,
    Read,
    Write,
    RenderTarget,
    DepthStencil,
}

#[derive(Clone)]
pub enum RenderGraphResource {
    Texture(Texture),
    Buffer(Buffer),
}

pub struct RenderGraphContext {
    resource_state: HashMap<(RenderGraphResourceId, u8), RenderGraphResourceState>,
    api_resource_state: HashMap<(RenderGraphResourceId, u8), ResourceState>,
    created: Vec<RenderGraphResourceId>,
    lifetimes: Vec<(*const RGNode, *const RGNode)>, // indexed by resourceId
    resources: Vec<Option<RenderGraphResource>>,    // indexed by resourceId
}

pub struct RenderGraphExecuteContext<'a> {
    pub(crate) read_resources: &'a Vec<(RenderGraphResourceId, RenderGraphViewId)>,
    pub(crate) write_resources: &'a Vec<(RenderGraphResourceId, RenderGraphViewId)>,
    pub(crate) render_targets: &'a Vec<Option<(RenderGraphResourceId, RenderGraphViewId)>>,
    pub(crate) depth_stencil: &'a Option<(RenderGraphResourceId, RenderGraphViewId)>,
    pub(crate) command_buffer: &'a mut CommandBuffer,
}

pub struct RenderGraph {
    pub(crate) root: RGNode,
    pub(crate) resources: Vec<RenderGraphResourceDef>,
    pub(crate) resource_names: Vec<String>,
    pub(crate) injected_resources: Vec<(RenderGraphResourceId, RenderGraphResource)>,
    pub(crate) views: Vec<RenderGraphViewDef>,
}

struct ResourceBarrier {
    resource_id: (RenderGraphResourceId, RenderGraphViewId),
    previous_state: RenderGraphResourceState,
    next_state: RenderGraphResourceState,
}

impl RenderGraph {
    pub(crate) fn builder() -> RenderGraphBuilder {
        RenderGraphBuilder::default()
    }

    #[allow(clippy::unused_self)]
    fn get_api_state(
        &self,
        state: RenderGraphResourceState,
        texture_view_def: Option<&RenderGraphTextureViewDef>,
    ) -> ResourceState {
        match state {
            RenderGraphResourceState::Unknown => ResourceState::UNDEFINED,
            RenderGraphResourceState::Read => ResourceState::SHADER_RESOURCE,
            RenderGraphResourceState::Write => ResourceState::UNORDERED_ACCESS,
            RenderGraphResourceState::RenderTarget => ResourceState::RENDER_TARGET,
            RenderGraphResourceState::DepthStencil => {
                if texture_view_def.unwrap().read_only {
                    ResourceState::DEPTH_READ
                } else {
                    ResourceState::DEPTH_WRITE
                }
            }
        }
    }

    fn create_texture(
        &self,
        resource_id: (RenderGraphResourceId, RenderGraphViewId),
        context: &mut RenderGraphContext,
        device_context: &DeviceContext,
    ) {
        let res_id = resource_id.0 as usize;

        // TODO: Create the resource, clear it if necessary, and transition it to write.
        if !context.created.iter().any(|r| *r == resource_id.0) {
            if !self.injected_resources.iter().any(|r| r.0 == resource_id.0) {
                println!("  !! Create {} ", self.resource_names[res_id]);
                let texture_def = &self.resources[res_id];
                let texture_def = texture_def.texture_def().clone();
                let texture = device_context.create_texture(&texture_def.clone().into());
                let texture = RenderGraphResource::Texture(texture);
                context.resources[res_id] = Some(texture);

                for mip in 0..texture_def.mip_count {
                    let res_mip_id = (res_id as u32, mip as u8);
                    context
                        .resource_state
                        .insert(res_mip_id, RenderGraphResourceState::Unknown);
                    context
                        .api_resource_state
                        .insert(res_mip_id, ResourceState::UNDEFINED);
                }
                // TODO: Create
            }

            context.created.push(resource_id.0);
            if self.resources[res_id].texture_def().clear {
                println!("  !! Clear {} ", self.resource_names[res_id]);
                // TODO: Clear
            }
        }
    }

    fn create_buffer(
        &self,
        resource_id: (RenderGraphResourceId, RenderGraphViewId),
        context: &mut RenderGraphContext,
        device_context: &DeviceContext,
    ) {
        let res_id = resource_id.0 as usize;

        // TODO: Create the resource, clear it if necessary, and transition it to write.
        if !context.created.iter().any(|r| *r == resource_id.0) {
            if !self.injected_resources.iter().any(|r| r.0 == resource_id.0) {
                println!("  !! Create {} ", self.resource_names[res_id]);
                let buffer_def = &self.resources[res_id];
                let buffer_def = buffer_def.buffer_def().clone();
                let buffer = device_context.create_buffer(&buffer_def.into());
                let buffer = RenderGraphResource::Buffer(buffer);
                context.resources[res_id] = Some(buffer);

                let res_mip_id = (res_id as u32, 0);
                context
                    .resource_state
                    .insert(res_mip_id, RenderGraphResourceState::Unknown);
                context
                    .api_resource_state
                    .insert(res_mip_id, ResourceState::UNDEFINED);
                // TODO: Create
            }

            context.created.push(resource_id.0);
            //if self.resources[res_id].buffer_def().clear {
            //    println!("  !! Clear {} ", self.resource_names[res_id]);
            //    // TODO: Clear
            //}
        }
    }

    fn transition_texture<'a>(
        &self,
        res_mip_id: (RenderGraphResourceId, u8),
        texture: &'a Texture,
        texture_view_def: &RenderGraphTextureViewDef,
        previous_state: RenderGraphResourceState,
        next_state: RenderGraphResourceState,
        texture_barriers: &mut Vec<TextureBarrier<'a>>,
    ) {
        println!(
            "  Transition texture {} mip {} from {:?} to {:?}",
            self.resource_names[res_mip_id.0 as usize],
            texture_view_def.first_mip,
            previous_state,
            next_state,
        );

        texture_barriers.push(TextureBarrier::state_transition_for_mip(
            texture,
            self.get_api_state(previous_state, Some(texture_view_def)),
            self.get_api_state(next_state, Some(texture_view_def)),
            Some(texture_view_def.first_mip as u8),
        ));
    }

    fn transition_buffer<'a>(
        &self,
        res_mip_id: (RenderGraphResourceId, u8),
        buffer: &'a Buffer,
        previous_state: RenderGraphResourceState,
        next_state: RenderGraphResourceState,
        buffer_barriers: &mut Vec<BufferBarrier<'a>>,
    ) {
        println!(
            "  Transition buffer {} from {:?} to {:?}",
            self.resource_names[res_mip_id.0 as usize], previous_state, next_state,
        );

        buffer_barriers.push(BufferBarrier {
            buffer,
            src_state: self.get_api_state(previous_state, None),
            dst_state: self.get_api_state(next_state, None),
            queue_transition: BarrierQueueTransition::default(),
        });
    }

    fn gather_texture_transitions(
        &self,
        resource_id: (RenderGraphResourceId, RenderGraphViewId),
        texture_view_def: &RenderGraphTextureViewDef,
        next_state: RenderGraphResourceState,
        context: &mut RenderGraphContext,
        device_context: &DeviceContext,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        let res_id = resource_id.0 as usize;

        // Create if needed
        let mip_0_id = (res_id as u32, 0);
        let mip_0_state = *context
            .resource_state
            .entry(mip_0_id)
            .or_insert(RenderGraphResourceState::Unknown);

        if mip_0_state == RenderGraphResourceState::Unknown {
            self.create_texture(resource_id, context, device_context);
        }

        assert!(
            context.resources[res_id].is_some(),
            "Resource {} should have been created before being transitioned.",
            self.resource_names[res_id]
        );

        // Gather transitions
        let first_mip = texture_view_def.first_mip;
        let mip_count = texture_view_def.mip_count;
        for mip in first_mip..first_mip + mip_count {
            let res_mip_id = (res_id as u32, mip as u8);

            let previous_state = context
                .resource_state
                .entry(res_mip_id)
                .or_insert(RenderGraphResourceState::Unknown);

            if *previous_state == next_state {
                // Nothing to do.
            } else {
                match context.resources[res_id].as_ref().unwrap() {
                    RenderGraphResource::Texture(_) => {
                        barriers.push(ResourceBarrier {
                            resource_id,
                            previous_state: *previous_state,
                            next_state,
                        });
                    }
                    RenderGraphResource::Buffer(_) => {
                        panic!("View was TextureView but Resource is Buffer?")
                    }
                }

                context.resource_state.insert(res_mip_id, next_state);
                context.api_resource_state.insert(
                    res_mip_id,
                    self.get_api_state(next_state, Some(texture_view_def)),
                );
            }
        }
    }

    fn gather_buffer_transitions(
        &self,
        resource_id: (RenderGraphResourceId, RenderGraphViewId),
        next_state: RenderGraphResourceState,
        context: &mut RenderGraphContext,
        device_context: &DeviceContext,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        let res_id = resource_id.0 as usize;

        // Create if needed
        let mip_0_id = (res_id as u32, 0);
        let mip_0_state = *context
            .resource_state
            .entry(mip_0_id)
            .or_insert(RenderGraphResourceState::Unknown);

        if mip_0_state == RenderGraphResourceState::Unknown {
            self.create_buffer(resource_id, context, device_context);
        }

        assert!(
            context.resources[res_id].is_some(),
            "Resource {} should have been created before being transitioned.",
            self.resource_names[res_id]
        );

        // Gather transitions
        let res_mip_id = (res_id as u32, 0);

        let previous_state = context
            .resource_state
            .entry(res_mip_id)
            .or_insert(RenderGraphResourceState::Unknown);

        if *previous_state == next_state {
            // Nothing to do.
        } else {
            match context.resources[res_id].as_ref().unwrap() {
                RenderGraphResource::Texture(_) => {
                    panic!("View was TextureView but Resource is Buffer?")
                }
                RenderGraphResource::Buffer(_) => {
                    barriers.push(ResourceBarrier {
                        resource_id,
                        previous_state: *previous_state,
                        next_state,
                    });
                }
            }

            context.resource_state.insert(res_mip_id, next_state);
            context
                .api_resource_state
                .insert(res_mip_id, self.get_api_state(next_state, None));
        }
    }

    fn gather_resource_transitions<'a>(
        &self,
        resource_id: (RenderGraphResourceId, RenderGraphViewId),
        next_state: RenderGraphResourceState,
        context: &'a mut RenderGraphContext,
        device_context: &DeviceContext,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        let view_id = resource_id.1 as usize;

        let view_def = &self.views[view_id];
        match view_def {
            RenderGraphViewDef::Texture(texture_view_def) => {
                self.gather_texture_transitions(
                    resource_id,
                    texture_view_def,
                    next_state,
                    context,
                    device_context,
                    barriers,
                );
            }
            RenderGraphViewDef::Buffer(_) => {
                self.gather_buffer_transitions(
                    resource_id,
                    next_state,
                    context,
                    device_context,
                    barriers,
                );
            }
        }
    }

    fn gather_read_resource_transitions(
        &self,
        context: &mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_>,
        device_context: &DeviceContext,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        for read_res in execute_context.read_resources {
            self.gather_resource_transitions(
                *read_res,
                RenderGraphResourceState::Read,
                context,
                device_context,
                barriers,
            );
        }
    }

    fn gather_write_resource_transitions<'a>(
        &self,
        context: &'a mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_>,
        device_context: &DeviceContext,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        for write_res in execute_context.write_resources {
            self.gather_resource_transitions(
                *write_res,
                RenderGraphResourceState::Write,
                context,
                device_context,
                barriers,
            );
        }
    }

    fn gather_rt_resource_transitions<'a>(
        &self,
        context: &'a mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_>,
        device_context: &DeviceContext,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        for rt_res in execute_context.render_targets.iter().flatten() {
            self.gather_resource_transitions(
                *rt_res,
                RenderGraphResourceState::RenderTarget,
                context,
                device_context,
                barriers,
            );
        }
    }

    fn gather_depth_stencil_resource_transitions<'a>(
        &self,
        context: &'a mut RenderGraphContext,
        execute_context: &RenderGraphExecuteContext<'_>,
        device_context: &DeviceContext,
        barriers: &mut Vec<ResourceBarrier>,
    ) {
        if let Some(depth_stencil_res) = execute_context.depth_stencil {
            self.gather_resource_transitions(
                *depth_stencil_res,
                RenderGraphResourceState::DepthStencil,
                context,
                device_context,
                barriers,
            );
        }
    }

    pub fn begin_execute(
        &self,
        context: &mut RenderGraphContext,
        execute_context: &mut RenderGraphExecuteContext<'_>,
        device_context: &DeviceContext,
    ) {
        let mut barriers: Vec<ResourceBarrier> = Vec::with_capacity(32);

        self.gather_read_resource_transitions(
            context,
            execute_context,
            device_context,
            &mut barriers,
        );

        self.gather_write_resource_transitions(
            context,
            execute_context,
            device_context,
            &mut barriers,
        );

        self.gather_rt_resource_transitions(
            context,
            execute_context,
            device_context,
            &mut barriers,
        );

        self.gather_depth_stencil_resource_transitions(
            context,
            execute_context,
            device_context,
            &mut barriers,
        );

        // Create the actual barriers
        let mut buffer_barriers: Vec<BufferBarrier<'_>> = Vec::with_capacity(32);
        let mut texture_barriers: Vec<TextureBarrier<'_>> = Vec::with_capacity(32);

        for barrier in &barriers {
            let res_id = barrier.resource_id.0 as usize;
            let view_id = barrier.resource_id.1 as usize;
            match context.resources[res_id].as_ref().unwrap() {
                RenderGraphResource::Texture(texture) => {
                    let texture_view_def = &self.views[view_id].texture_view_def();
                    let first_mip = texture_view_def.first_mip;
                    let mip_count = texture_view_def.mip_count;
                    for mip in first_mip..first_mip + mip_count {
                        let res_mip_id = (res_id as u32, mip as u8);
                        self.transition_texture(
                            res_mip_id,
                            texture,
                            texture_view_def,
                            barrier.previous_state,
                            barrier.next_state,
                            &mut texture_barriers,
                        );
                    }
                }
                RenderGraphResource::Buffer(buffer) => {
                    let res_mip_id = (res_id as u32, 0);
                    self.transition_buffer(
                        res_mip_id,
                        buffer,
                        barrier.previous_state,
                        barrier.next_state,
                        &mut buffer_barriers,
                    );
                }
            }
        }

        execute_context.command_buffer.begin().unwrap();
        // Execute the batch of barriers.
        let execute_barriers = true;
        if execute_barriers {
            execute_context
                .command_buffer
                .cmd_resource_barrier(&buffer_barriers, &texture_barriers);
        }
        execute_context.command_buffer.end().unwrap();
    }

    fn end_execute(
        &self,
        context: &RenderGraphContext,
        node: &RGNode,
        _device_context: &DeviceContext,
    ) {
        for (res_id, lifetime) in context.lifetimes.iter().enumerate() {
            if lifetime.1 == node && !self.injected_resources.iter().any(|r| r.0 == res_id as u32) {
                // TODO: Deallocate resource
                println!("  !! Destroy {}", self.resource_names[res_id]);
            }
        }
    }

    pub fn compile(&self) -> RenderGraphContext {
        let mut context = RenderGraphContext {
            resource_state: HashMap::with_capacity(self.resources.len()),
            api_resource_state: HashMap::with_capacity(self.resources.len()),
            created: vec![],
            lifetimes: Vec::with_capacity(self.resources.len()),
            resources: vec![None; self.resources.len()],
        };

        // Add injected resources since they are already created (outside the graph)
        for injected_resource in &self.injected_resources {
            context.resources[injected_resource.0 as usize] = Some(injected_resource.1.clone());
        }

        for (id, res) in self.resources.iter().enumerate() {
            context
                .lifetimes
                .push(self.find_lifetime_start_and_end(id as u32, res));
        }

        context
    }

    pub fn execute(
        &self,
        context: &mut RenderGraphContext,
        device_context: &DeviceContext,
        command_pool: &CommandPool,
    ) {
        self.execute_inner(context, &self.root, device_context, command_pool);
    }

    fn execute_inner(
        &self,
        context: &mut RenderGraphContext,
        node: &RGNode,
        device_context: &DeviceContext,
        command_pool: &CommandPool,
    ) {
        if let Some(execute_fn) = &node.execute_fn {
            println!("--- Executing {}", node.name);
            let mut command_buffer = command_pool
                .create_command_buffer(&CommandBufferDef {
                    is_secondary: false,
                })
                .unwrap();

            command_buffer.begin_label(&node.name);

            {
                let mut execute_context = RenderGraphExecuteContext {
                    read_resources: &node.read_resources,
                    write_resources: &node.write_resources,
                    render_targets: &node.render_targets,
                    depth_stencil: &node.depth_stencil,
                    command_buffer: &mut command_buffer,
                };

                self.begin_execute(context, &mut execute_context, device_context);
                (execute_fn)(&execute_context);
                self.end_execute(context, node, device_context);
            }

            command_buffer.end_label();
        }

        for child in &node.children {
            self.execute_inner(context, child, device_context, command_pool);
        }
    }

    fn find_lifetime_start_and_end(
        &self,
        id: u32,
        res: &RenderGraphResourceDef,
    ) -> (*const RGNode, *const RGNode) {
        let mut first_node: Option<&RGNode> = None;
        let mut last_node: Option<&RGNode> = None;

        self.find_lifetime_start_and_end_inner(
            id,
            res,
            &self.root,
            &mut first_node,
            &mut last_node,
        );

        let injected = self.injected_resources.iter().any(|r| r.0 == id);

        println!(
            "Resource {} first_node {} last_node {} {}",
            self.resource_names[id as usize],
            first_node.unwrap().name,
            last_node.unwrap().name,
            if injected { "(injected)" } else { "" },
        );

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
        let resource_used = node
            .read_resources
            .iter()
            .any(|res_and_view| res_and_view.0 == id);
        let resource_used = resource_used
            || node
                .write_resources
                .iter()
                .any(|res_and_view| res_and_view.0 == id);
        let resource_used = resource_used
            || node
                .render_targets
                .iter()
                .any(|res_and_view| match res_and_view {
                    Some(res_and_view) => res_and_view.0 == id,
                    _ => false,
                });
        let resource_used = resource_used
            || match node.depth_stencil {
                Some(res_and_view) => res_and_view.0 == id,
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
        let printed = self
            .root
            .print(0, &self.resources, &self.resource_names, &self.views);
        write!(f, "{}", printed)
    }
}
