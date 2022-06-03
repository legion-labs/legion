use lgn_graphics_api::{CommandBuffer, DeviceContext, Format, GPUViewType, ResourceState, Texture};

use crate::{
    core::{
        render_graph::{
            RGNode, RenderGraph, RenderGraphExecuteContext, RenderGraphResource,
            RenderGraphResourceDef, RenderGraphResourceId, RenderGraphViewDef, RenderGraphViewId,
        },
        RenderResources,
    },
    resources::PipelineManager,
};

use super::{RenderGraphContext, RenderGraphLoadState, ResourceData};

pub(crate) struct GraphicsPassBuilder {
    node: RGNode,
}

impl GraphicsPassBuilder {
    #[allow(dead_code)]
    pub fn read(mut self, view: RenderGraphViewId, resource_state: RenderGraphLoadState) -> Self {
        self.node.read_resources.push(ResourceData {
            key: view,
            load_state: resource_state,
        });
        self
    }

    #[allow(dead_code)]
    pub fn write(mut self, view: RenderGraphViewId, resource_state: RenderGraphLoadState) -> Self {
        self.node.write_resources.push(ResourceData {
            key: view,
            load_state: resource_state,
        });
        self
    }

    pub fn render_target(
        mut self,
        slot: u32,
        view: RenderGraphViewId,
        resource_state: RenderGraphLoadState,
    ) -> Self {
        self.node.render_targets[slot as usize] = Some(ResourceData {
            key: view,
            load_state: resource_state,
        });
        self
    }

    pub fn depth_stencil(
        mut self,
        view: RenderGraphViewId,
        resource_state: RenderGraphLoadState,
    ) -> Self {
        self.node.depth_stencil = Some(ResourceData {
            key: view,
            load_state: resource_state,
        });
        self
    }

    pub fn execute<F: 'static>(mut self, f: F) -> Self
    where
        F: Fn(&RenderGraphContext, &mut RenderGraphExecuteContext<'_, '_>, &mut CommandBuffer),
    {
        self.node.execute_fn = Some(Box::new(f));
        self
    }
}

pub(crate) struct ComputePassBuilder {
    node: RGNode,
}

impl ComputePassBuilder {
    pub fn read(mut self, view: RenderGraphViewId, resource_state: RenderGraphLoadState) -> Self {
        self.node.read_resources.push(ResourceData {
            key: view,
            load_state: resource_state,
        });
        self
    }

    pub fn write(mut self, view: RenderGraphViewId, resource_state: RenderGraphLoadState) -> Self {
        self.node.write_resources.push(ResourceData {
            key: view,
            load_state: resource_state,
        });
        self
    }

    pub fn execute<F: 'static>(mut self, f: F) -> Self
    where
        F: Fn(&RenderGraphContext, &mut RenderGraphExecuteContext<'_, '_>, &mut CommandBuffer),
    {
        self.node.execute_fn = Some(Box::new(f));
        self
    }
}

pub(crate) struct RenderGraphBuilder<'a> {
    pub(crate) current_parent: Option<RGNode>,
    pub(crate) resources: Vec<RenderGraphResourceDef>,
    pub(crate) resource_names: Vec<String>,
    pub(crate) injected_resources:
        Vec<(RenderGraphResourceId, (RenderGraphResource, ResourceState))>,
    pub(crate) next_resource_id: RenderGraphResourceId,
    pub(crate) views: Vec<RenderGraphViewDef>,
    pub(crate) next_view_id: RenderGraphViewId,
    pub(crate) top_level_nodes: Vec<RGNode>,

    // Stuff used to initialize pass-specific user data when building render passes.
    // Should not be stored anywhere, they are made accessible in the execute functions anyways.
    pub(crate) render_resources: &'a RenderResources,
    pub(crate) pipeline_manager: &'a mut PipelineManager,
    pub(crate) device_context: &'a DeviceContext,
}

impl<'a> RenderGraphBuilder<'a> {
    pub fn new(
        render_resources: &'a RenderResources,
        pipeline_manager: &'a mut PipelineManager,
        device_context: &'a DeviceContext,
    ) -> Self {
        RenderGraphBuilder {
            current_parent: None,
            resources: vec![],
            resource_names: vec![],
            injected_resources: vec![],
            next_resource_id: 0,
            views: vec![],
            next_view_id: 0,
            top_level_nodes: vec![],
            render_resources,
            pipeline_manager,
            device_context,
        }
    }

    pub fn declare_render_target_with_def(
        &mut self,
        name: &str,
        resource: &RenderGraphResourceDef,
    ) -> RenderGraphResourceId {
        assert!(
            !self.resource_names.iter().any(|x| x == name),
            "Resource with the name {} already declared in this render graph.",
            name
        );

        let id = self.next_resource_id;
        self.next_resource_id += 1;
        self.resources.push(resource.clone());
        self.resource_names.push(name.to_string());
        id
    }

    pub fn declare_render_target(
        &mut self,
        name: &str,
        width: u32,
        height: u32,
        format: Format,
    ) -> RenderGraphResourceId {
        self.declare_render_target_with_def(
            name,
            &RenderGraphResourceDef::new_texture(width, height, 1, 1, 1, format),
        )
    }

    pub fn inject_render_target(
        &mut self,
        name: &str,
        texture: &Texture,
        initial_state: ResourceState,
    ) -> RenderGraphResourceId {
        let texture_def = *texture.definition();
        let texture_def = texture_def.into();
        let id = self
            .declare_render_target_with_def(name, &RenderGraphResourceDef::Texture(texture_def));
        self.injected_resources.push((
            id,
            (RenderGraphResource::Texture(texture.clone()), initial_state),
        ));
        id
    }

    pub fn declare_buffer_with_def(
        &mut self,
        name: &str,
        resource: &RenderGraphResourceDef,
    ) -> RenderGraphResourceId {
        assert!(
            !self.resource_names.iter().any(|x| x == name),
            "Resource with the name {} already declared in this render graph.",
            name
        );

        let id = self.next_resource_id;
        self.next_resource_id += 1;
        self.resources.push(resource.clone());
        self.resource_names.push(name.to_string());
        id
    }

    pub fn declare_buffer(
        &mut self,
        name: &str,
        element_size: u64,
        element_count: u64,
    ) -> RenderGraphResourceId {
        self.declare_buffer_with_def(
            name,
            &RenderGraphResourceDef::new_buffer(element_size, element_count),
        )
    }

    pub fn declare_view_with_def(&mut self, view: &RenderGraphViewDef) -> RenderGraphViewId {
        if let Some(index) = self.views.iter().position(|v| v == view) {
            index as RenderGraphViewId
        } else {
            let id = self.next_view_id;
            self.next_view_id += 1;
            self.views.push(view.clone());
            id
        }
    }

    pub fn declare_texture_srv(&mut self, resource_id: RenderGraphResourceId) -> RenderGraphViewId {
        self.declare_view_with_def(&RenderGraphViewDef::new_texture_view_with_mips(
            resource_id,
            0,
            1,
            GPUViewType::ShaderResource,
            false,
        ))
    }

    pub fn declare_texture_rtv(&mut self, resource_id: RenderGraphResourceId) -> RenderGraphViewId {
        self.declare_view_with_def(&RenderGraphViewDef::new_texture_view_with_mips(
            resource_id,
            0,
            1,
            GPUViewType::RenderTarget,
            false,
        ))
    }

    pub fn declare_texture_uav(&mut self, resource_id: RenderGraphResourceId) -> RenderGraphViewId {
        self.declare_view_with_def(&RenderGraphViewDef::new_texture_view_with_mips(
            resource_id,
            0,
            1,
            GPUViewType::UnorderedAccess,
            false,
        ))
    }

    pub fn declare_texture_rtv_with_mips(
        &mut self,
        resource_id: RenderGraphResourceId,
        first_mip: u32,
        mip_count: u32,
    ) -> RenderGraphViewId {
        self.declare_view_with_def(&RenderGraphViewDef::new_texture_view_with_mips(
            resource_id,
            first_mip,
            mip_count,
            GPUViewType::RenderTarget,
            false,
        ))
    }

    pub fn declare_texture_srv_with_mips(
        &mut self,
        resource_id: RenderGraphResourceId,
        first_mip: u32,
        mip_count: u32,
    ) -> RenderGraphViewId {
        self.declare_view_with_def(&RenderGraphViewDef::new_texture_view_with_mips(
            resource_id,
            first_mip,
            mip_count,
            GPUViewType::ShaderResource,
            false,
        ))
    }

    pub fn declare_depth_texture_dsv(
        &mut self,
        resource_id: RenderGraphResourceId,
        read_only: bool,
    ) -> RenderGraphViewId {
        self.declare_view_with_def(&RenderGraphViewDef::new_depth_texture_view(
            resource_id,
            0,
            GPUViewType::DepthStencil,
            read_only,
        ))
    }

    pub fn declare_depth_texture_srv(
        &mut self,
        resource_id: RenderGraphResourceId,
    ) -> RenderGraphViewId {
        self.declare_view_with_def(&RenderGraphViewDef::new_depth_texture_view(
            resource_id,
            0,
            GPUViewType::ShaderResource,
            false,
        ))
    }

    pub fn declare_buffer_srv(&mut self, resource_id: RenderGraphResourceId) -> RenderGraphViewId {
        let resource_def = self.resources[resource_id as usize].clone();
        self.declare_view_with_def(&RenderGraphViewDef::new_srv_buffer_view(
            resource_id,
            &resource_def,
        ))
    }

    pub fn declare_buffer_uav(&mut self, resource_id: RenderGraphResourceId) -> RenderGraphViewId {
        let resource_def = self.resources[resource_id as usize].clone();
        self.declare_view_with_def(&RenderGraphViewDef::new_uav_buffer_view(
            resource_id,
            &resource_def,
        ))
    }

    pub fn declare_buffer_indirect(
        &mut self,
        resource_id: RenderGraphResourceId,
    ) -> RenderGraphViewId {
        let resource_def = self.resources[resource_id as usize].clone();
        self.declare_view_with_def(&RenderGraphViewDef::new_indirect_buffer_view(
            resource_id,
            &resource_def,
        ))
    }

    pub fn declare_buffer_copy_dst(
        &mut self,
        resource_id: RenderGraphResourceId,
    ) -> RenderGraphViewId {
        let resource_def = self.resources[resource_id as usize].clone();
        self.declare_view_with_def(&RenderGraphViewDef::new_copy_dst_buffer_view(
            resource_id,
            &resource_def,
        ))
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

        RenderGraph {
            root_nodes: self.top_level_nodes,
            resource_defs: self.resources,
            resource_names: self.resource_names,
            injected_resources: self.injected_resources,
            view_defs: self.views,
        }
    }
}
