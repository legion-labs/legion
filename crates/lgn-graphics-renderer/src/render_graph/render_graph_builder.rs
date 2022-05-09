use lgn_graphics_api::{ColorClearValue, DepthStencilClearValue, Texture};

use crate::{
    hl_gfx_api::HLCommandBuffer,
    render_graph::{
        RGNode, RenderGraph, RenderGraphExecuteContext, RenderGraphResource,
        RenderGraphResourceDef, RenderGraphResourceId, RenderGraphViewDef, RenderGraphViewId,
    },
};

pub(crate) struct GraphicsPassBuilder {
    node: RGNode,
}

impl GraphicsPassBuilder {
    #[allow(dead_code)]
    pub fn read(mut self, resource: RenderGraphResourceId, view: RenderGraphViewId) -> Self {
        self.node.read_resources.push((resource, view));
        self
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

    pub fn clear_rt(mut self, slot: u32, clear_value: ColorClearValue) -> Self {
        self.node.clear_rt_resources[slot as usize] = Some(clear_value);
        self
    }

    pub fn clear_ds(mut self, clear_value: DepthStencilClearValue) -> Self {
        self.node.clear_ds_resource = Some(clear_value);
        self
    }

    #[allow(dead_code)]
    pub fn clear_write(mut self, slot: u32, clear_value: u32) -> Self {
        if self.node.clear_write_resources.len() <= slot as usize {
            self.node
                .clear_write_resources
                .resize(slot as usize + 1, None);
        }
        self.node.clear_write_resources[slot as usize] = Some(clear_value);
        self
    }

    pub fn execute<F>(mut self, f: F) -> Self
    where
        F: Fn(&RenderGraphExecuteContext<'_>, &mut HLCommandBuffer<'_>) + 'static,
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

    pub fn clear_write(mut self, slot: u32, clear_value: u32) -> Self {
        if self.node.clear_write_resources.len() <= slot as usize {
            self.node
                .clear_write_resources
                .resize(slot as usize + 1, None);
        }
        self.node.clear_write_resources[slot as usize] = Some(clear_value);
        self
    }

    pub fn execute<F>(mut self, f: F) -> Self
    where
        F: Fn(&RenderGraphExecuteContext<'_>, &mut HLCommandBuffer<'_>) + 'static,
    {
        self.node.execute_fn = Some(Box::new(f));
        self
    }
}

#[derive(Default)]
pub(crate) struct RenderGraphBuilder {
    pub(crate) current_parent: Option<RGNode>,
    pub(crate) resources: Vec<RenderGraphResourceDef>,
    pub(crate) resource_names: Vec<String>,
    pub(crate) injected_resources: Vec<(RenderGraphResourceId, RenderGraphResource)>,
    pub(crate) next_resource_id: RenderGraphResourceId,
    pub(crate) views: Vec<RenderGraphViewDef>,
    pub(crate) next_view_id: RenderGraphViewId,
    pub(crate) top_level_nodes: Vec<RGNode>,
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

    pub fn inject_render_target(&mut self, name: &str, texture: &Texture) -> RenderGraphResourceId {
        let id = self.declare_render_target(
            name,
            &RenderGraphResourceDef::Texture(texture.definition().clone().into()),
        );
        self.injected_resources
            .push((id, RenderGraphResource::Texture(texture.clone())));
        id
    }

    pub fn declare_view(&mut self, view: &RenderGraphViewDef) -> RenderGraphViewId {
        if let Some(index) = self.views.iter().position(|v| v == view) {
            index as RenderGraphViewId
        } else {
            let id = self.next_view_id;
            self.next_view_id += 1;
            self.views.push(view.clone());
            id
        }
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
            children: self.top_level_nodes,
            ..RGNode::default()
        };

        RenderGraph {
            root,
            resources: self.resources,
            resource_names: self.resource_names,
            injected_resources: self.injected_resources,
            views: self.views,
        }
    }
}
