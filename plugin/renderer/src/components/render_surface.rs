use std::{cmp::max, sync::Arc};

use lgn_ecs::prelude::Component;
use lgn_graphics_api::{
    Extents2D, Extents3D, Format, MemoryUsage, ResourceFlags, ResourceState, ResourceUsage,
    Semaphore, Texture, TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef,
};
use lgn_tasks::TaskPool;
use lgn_window::WindowId;
use parking_lot::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

use crate::egui::egui_pass::EguiPass;
use crate::hl_gfx_api::HLCommandBuffer;
use crate::render_pass::{DebugRenderPass, PickingRenderPass, TmpRenderPass};
use crate::{RenderContext, Renderer};

pub trait Presenter: Send + Sync {
    fn resize(&mut self, renderer: &Renderer, extents: RenderSurfaceExtents);
    fn present(
        &mut self,
        render_context: &RenderContext<'_>,
        render_surface: &mut RenderSurface,
        task_pool: &TaskPool,
    );
}

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RenderSurfaceId(Uuid);

impl RenderSurfaceId {
    fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderSurfaceExtents {
    extents: Extents2D,
}

impl RenderSurfaceExtents {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            extents: Extents2D {
                width: max(1u32, width),
                height: max(1u32, height),
            },
        }
    }

    pub fn width(self) -> u32 {
        self.extents.width
    }

    pub fn height(self) -> u32 {
        self.extents.height
    }
}

pub struct RenderSurfaces {
    window_id_mapper: HashMap<WindowId, RenderSurfaceId>,
}

impl RenderSurfaces {
    pub fn new() -> Self {
        Self {
            window_id_mapper: HashMap::new(),
        }
    }

    pub fn insert(&mut self, window_id: WindowId, render_surface_id: RenderSurfaceId) {
        let result = self.window_id_mapper.insert(window_id, render_surface_id);
        assert!(result.is_none());
    }

    pub fn remove(&mut self, window_id: WindowId) {
        let result = self.window_id_mapper.remove(&window_id);
        assert!(result.is_some());
    }

    pub fn get_from_window_id(&self, window_id: WindowId) -> Option<&RenderSurfaceId> {
        self.window_id_mapper.get(&window_id)
    }
}

/// An event that is sent whenever a render surface is created for a window
#[derive(Debug, Clone)]
pub struct RenderSurfaceCreatedForWindow {
    pub window_id: WindowId,
    pub render_surface_id: RenderSurfaceId,
}

#[allow(dead_code)]
struct SizeDependentResources {
    texture: Texture,
    texture_srv: TextureView,
    texture_rtv: TextureView,
    texture_state: ResourceState,
    depth_stencil_texture: Texture,
    depth_stencil_texture_view: TextureView,
}

impl SizeDependentResources {
    fn new(renderer: &Renderer, extents: RenderSurfaceExtents) -> Self {
        let device_context = renderer.device_context();
        let texture_def = TextureDef {
            extents: Extents3D {
                width: extents.width(),
                height: extents.height(),
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R16G16B16A16_SFLOAT,
            usage_flags: ResourceUsage::AS_RENDER_TARGET
                | ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };
        let texture = device_context.create_texture(&texture_def).unwrap();

        let srv_def = TextureViewDef::as_shader_resource_view(&texture_def);
        let texture_srv = texture.create_view(&srv_def).unwrap();

        let rtv_def = TextureViewDef::as_render_target_view(&texture_def);
        let texture_rtv = texture.create_view(&rtv_def).unwrap();

        let depth_stencil_def = TextureDef {
            extents: Extents3D {
                width: extents.width(),
                height: extents.height(),
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::D32_SFLOAT,
            usage_flags: ResourceUsage::AS_DEPTH_STENCIL,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };

        let depth_stencil_texture = device_context.create_texture(&depth_stencil_def).unwrap();
        let depth_stencil_texture_view_def =
            TextureViewDef::as_depth_stencil_view(&depth_stencil_def);
        let depth_stencil_texture_view = depth_stencil_texture
            .create_view(&depth_stencil_texture_view_def)
            .unwrap();

        Self {
            texture,
            texture_srv,
            texture_rtv,
            texture_state: ResourceState::UNDEFINED,
            depth_stencil_texture,
            depth_stencil_texture_view,
        }
    }
}

#[derive(Component)]
pub struct RenderSurface {
    id: RenderSurfaceId,
    extents: RenderSurfaceExtents,
    resources: SizeDependentResources,
    presenters: Vec<Box<dyn Presenter>>,
    // tmp
    num_render_frames: usize,
    render_frame_idx: usize,
    signal_sems: Vec<Semaphore>,
    picking_renderpass: Arc<RwLock<PickingRenderPass>>,
    debug_renderpass: Arc<RwLock<DebugRenderPass>>,
    test_renderpass: Arc<RwLock<TmpRenderPass>>,
    egui_renderpass: Arc<RwLock<EguiPass>>,
}

impl RenderSurface {
    pub fn new(renderer: &Renderer, extents: RenderSurfaceExtents) -> Self {
        Self::new_with_id(RenderSurfaceId::new(), renderer, extents)
    }

    pub fn extents(&self) -> RenderSurfaceExtents {
        self.extents
    }

    pub fn picking_renderpass(&self) -> Arc<RwLock<PickingRenderPass>> {
        self.picking_renderpass.clone()
    }

    pub fn test_renderpass(&self) -> Arc<RwLock<TmpRenderPass>> {
        self.test_renderpass.clone()
    }

    pub fn debug_renderpass(&self) -> Arc<RwLock<DebugRenderPass>> {
        self.debug_renderpass.clone()
    }

    pub fn egui_renderpass(&self) -> Arc<RwLock<EguiPass>> {
        self.egui_renderpass.clone()
    }

    pub fn resize(&mut self, renderer: &Renderer, extents: RenderSurfaceExtents) {
        if self.extents != extents {
            self.resources = SizeDependentResources::new(renderer, extents);
            for presenter in &mut self.presenters {
                presenter.resize(renderer, extents);
            }
            self.extents = extents;
        }
    }

    pub fn register_presenter<T: 'static + Presenter>(&mut self, create_fn: impl FnOnce() -> T) {
        let presenter = create_fn();
        self.presenters.push(Box::new(presenter));
    }

    pub fn id(&self) -> RenderSurfaceId {
        self.id
    }

    pub fn texture(&self) -> &Texture {
        &self.resources.texture
    }

    pub fn render_target_view(&self) -> &TextureView {
        &self.resources.texture_rtv
    }

    pub fn shader_resource_view(&self) -> &TextureView {
        &self.resources.texture_srv
    }

    pub fn depth_stencil_texture_view(&self) -> &TextureView {
        &self.resources.depth_stencil_texture_view
    }

    pub fn transition_to(&mut self, cmd_buffer: &HLCommandBuffer<'_>, dst_state: ResourceState) {
        let src_state = self.resources.texture_state;
        let dst_state = dst_state;

        if src_state != dst_state {
            cmd_buffer.resource_barrier(
                &[],
                &[TextureBarrier::state_transition(
                    &self.resources.texture,
                    src_state,
                    dst_state,
                )],
            );
            self.resources.texture_state = dst_state;
        }
    }

    pub fn present(&mut self, render_context: &RenderContext<'_>, task_pool: &TaskPool) {
        let mut presenters = std::mem::take(&mut self.presenters);

        for presenter in &mut presenters {
            presenter.as_mut().present(render_context, self, task_pool);
        }

        self.presenters = presenters;
    }

    //
    // TODO: change that asap. Acquire can't be called more than once per frame.
    // This would result in a crash.
    //
    pub fn acquire(&mut self) -> &Semaphore {
        let render_frame_idx = (self.render_frame_idx + 1) % self.num_render_frames;
        let sem = &self.signal_sems[render_frame_idx];
        self.render_frame_idx = render_frame_idx;
        sem
    }

    pub fn sema(&self) -> &Semaphore {
        &self.signal_sems[self.render_frame_idx]
    }

    fn new_with_id(
        id: RenderSurfaceId,
        renderer: &Renderer,
        extents: RenderSurfaceExtents,
    ) -> Self {
        let num_render_frames = renderer.num_render_frames();
        let device_context = renderer.device_context();
        let signal_sems = (0..num_render_frames)
            .map(|_| device_context.create_semaphore())
            .collect();

        Self {
            id,
            extents,
            resources: SizeDependentResources::new(renderer, extents),
            num_render_frames,
            render_frame_idx: 0,
            signal_sems,
            picking_renderpass: Arc::new(RwLock::new(PickingRenderPass::new(renderer))),
            test_renderpass: Arc::new(RwLock::new(TmpRenderPass::new(renderer))),
            debug_renderpass: Arc::new(RwLock::new(DebugRenderPass::new(renderer))),
            egui_renderpass: Arc::new(RwLock::new(EguiPass::new(renderer))),
            presenters: Vec::new(),
        }
    }
}
