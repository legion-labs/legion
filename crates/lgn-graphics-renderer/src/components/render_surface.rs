use std::collections::hash_map::{Values, ValuesMut};
use std::{cmp::max, sync::Arc};

use lgn_graphics_api::{
    ColorClearValue, ColorRenderTargetBinding, CommandBuffer, DepthStencilClearValue,
    DepthStencilRenderTargetBinding, DeviceContext, Extents2D, Extents3D, Format, GPUViewType,
    LoadOp, MemoryUsage, PlaneSlice, ResourceFlags, ResourceState, ResourceUsage, Semaphore,
    SemaphoreDef, StoreOp, Texture, TextureDef, TextureTiling, TextureViewDef, ViewDimension,
};
use lgn_window::WindowId;
use parking_lot::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

use crate::egui::egui_pass::EguiPass;
use crate::gpu_renderer::HzbSurface;
use crate::render_pass::{
    DebugRenderPass, FinalResolveRenderPass, PickingRenderPass, RenderTarget,
};
use crate::resources::PipelineManager;
use crate::{RenderContext, Renderer};

pub trait Presenter: Send + Sync {
    fn resize(&mut self, device_context: &DeviceContext, extents: RenderSurfaceExtents);
    fn present(
        &mut self,
        render_context: &mut RenderContext<'_>,
        render_surface: &mut RenderSurface,
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

pub struct RenderSurfaceIterator<'a> {
    values: Values<'a, RenderSurfaceId, Box<RenderSurface>>,
}

impl<'a> Iterator for RenderSurfaceIterator<'a> {
    type Item = &'a RenderSurface;

    fn next(&mut self) -> Option<Self::Item> {
        self.values.next().map(std::convert::AsRef::as_ref)
    }
}

pub struct RenderSurfaceIteratorMut<'a> {
    values: ValuesMut<'a, RenderSurfaceId, Box<RenderSurface>>,
}

impl<'a> Iterator for RenderSurfaceIteratorMut<'a> {
    type Item = &'a mut RenderSurface;

    fn next(&mut self) -> Option<Self::Item> {
        self.values.next().map(std::convert::AsMut::as_mut)
    }
}

pub struct RenderSurfaces {
    surfaces: HashMap<RenderSurfaceId, Box<RenderSurface>>,
    window_id_mapper: HashMap<WindowId, RenderSurfaceId>,
}

impl RenderSurfaces {
    pub fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
            window_id_mapper: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.surfaces.clear();
        self.window_id_mapper.clear();
    }

    pub fn insert(&mut self, render_surface: RenderSurface) {
        let id = render_surface.id();
        let window_id = render_surface.window_id();
        assert!(!self.surfaces.contains_key(&id));
        self.surfaces.insert(id, Box::new(render_surface));
        if let Some(window_id) = window_id {
            self.window_id_mapper.insert(window_id, id);
        }
    }

    pub fn remove_from_window_id(&mut self, window_id: WindowId) {
        let id = self.window_id_mapper.remove(&window_id);
        if let Some(id) = id {
            self.surfaces.remove(&id);
        };
    }

    pub fn get_from_window_id(&self, window_id: WindowId) -> &RenderSurface {
        let id = self.window_id_mapper.get(&window_id).unwrap();
        let surface = self.surfaces.get(id).unwrap();
        surface.as_ref()
    }

    pub fn try_get_from_window_id(&self, window_id: WindowId) -> Option<&RenderSurface> {
        self.window_id_mapper
            .get(&window_id)
            .map(|x| self.surfaces.get(x).unwrap().as_ref())
    }

    pub fn get_from_window_id_mut(&mut self, window_id: WindowId) -> &mut RenderSurface {
        let id = self.window_id_mapper.get(&window_id).unwrap();
        let surface = self.surfaces.get_mut(id).unwrap();
        surface.as_mut()
    }

    pub fn try_get_from_window_id_mut(
        &mut self,
        window_id: WindowId,
    ) -> Option<&mut RenderSurface> {
        self.window_id_mapper
            .get(&window_id)
            .map(|x| self.surfaces.get_mut(x).unwrap().as_mut())
    }

    pub fn for_each(&self, func: impl Fn(&RenderSurface)) {
        self.surfaces.iter().for_each(|(_, render_surface)| {
            func(render_surface.as_ref());
        });
    }

    pub fn for_each_mut(&mut self, func: impl Fn(&mut RenderSurface)) {
        self.surfaces.iter_mut().for_each(|(_, render_surface)| {
            func(render_surface.as_mut());
        });
    }

    pub fn iter(&self) -> RenderSurfaceIterator<'_> {
        RenderSurfaceIterator {
            values: self.surfaces.values(),
        }
    }

    pub fn iter_mut(&mut self) -> RenderSurfaceIteratorMut<'_> {
        RenderSurfaceIteratorMut {
            values: self.surfaces.values_mut(),
        }
    }
}

/// An event that is sent whenever a render surface is created for a window
#[derive(Debug, Clone)]
pub struct RenderSurfaceCreatedForWindow {
    pub window_id: WindowId,
}

#[allow(dead_code)]
struct SizeDependentResources {
    hdr_rt: RenderTarget,
    depth_rt: RenderTarget,
    hzb_surface: HzbSurface,
    hzb_init: bool,
}

impl SizeDependentResources {
    fn new(
        device_context: &DeviceContext,
        extents: RenderSurfaceExtents,
        pipeline_manager: &PipelineManager,
    ) -> Self {
        Self {
            hdr_rt: RenderTarget::new(
                device_context,
                "HDR_RT",
                extents,
                Format::R16G16B16A16_SFLOAT,
                ResourceUsage::AS_RENDER_TARGET
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_TRANSFERABLE,
                GPUViewType::RenderTarget,
            ),
            depth_rt: RenderTarget::new(
                device_context,
                "Depth_RT",
                extents,
                Format::D32_SFLOAT,
                ResourceUsage::AS_DEPTH_STENCIL | ResourceUsage::AS_SHADER_RESOURCE,
                GPUViewType::DepthStencil,
            ),
            hzb_surface: HzbSurface::new(device_context, extents, pipeline_manager),
            hzb_init: false,
        }
    }
}

pub enum RenderSurfacePresentingStatus {
    Presenting,
    Paused,
}

pub struct RenderSurface {
    id: RenderSurfaceId,
    window_id: Option<WindowId>,
    extents: RenderSurfaceExtents,
    resources: SizeDependentResources,
    presenters: Vec<Box<dyn Presenter>>,
    // tmp
    num_render_frames: u64,
    render_frame_idx: u64,
    presenter_semaphores: Vec<Semaphore>,
    picking_renderpass: Arc<RwLock<PickingRenderPass>>,
    debug_renderpass: Arc<RwLock<DebugRenderPass>>,
    egui_renderpass: Arc<RwLock<EguiPass>>,
    final_resolve_render_pass: Arc<RwLock<FinalResolveRenderPass>>,
    presenting_status: RenderSurfacePresentingStatus,

    // For render graph
    view_target: Texture,
    hzb: [Texture; 2],
    hzb_cleared: bool,
    use_view_target: bool,
}

impl RenderSurface {
    pub fn new(
        window_id: WindowId,
        renderer: &Renderer,
        pipeline_manager: &PipelineManager,
        render_surface_extents: RenderSurfaceExtents,
    ) -> Self {
        Self::new_internal(
            Some(window_id),
            renderer,
            pipeline_manager,
            render_surface_extents,
        )
    }

    pub fn new_offscreen_window(
        renderer: &Renderer,
        pipeline_manager: &PipelineManager,
        render_surface_extents: RenderSurfaceExtents,
    ) -> Self {
        Self::new_internal(None, renderer, pipeline_manager, render_surface_extents)
    }

    fn new_internal(
        window_id: Option<WindowId>,
        renderer: &Renderer,
        pipeline_manager: &PipelineManager,
        render_surface_extents: RenderSurfaceExtents,
    ) -> Self {
        let num_render_frames = renderer.num_render_frames();
        let device_context = renderer.device_context();
        let presenter_semaphores = (0..num_render_frames)
            .map(|_| device_context.create_semaphore(SemaphoreDef::default()))
            .collect();

        let extents = Extents3D {
            width: render_surface_extents.width(),
            height: render_surface_extents.height(),
            depth: 1,
        };
        let view_desc = TextureDef {
            extents,
            array_length: 1,
            mip_count: 1,
            format: Format::B8G8R8A8_UNORM,
            usage_flags: ResourceUsage::AS_RENDER_TARGET
                | ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_UNORDERED_ACCESS
                | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };
        let view_target = device_context.create_texture(view_desc, "ViewBuffer");

        let hzb_desc = Self::make_hzb_desc(&extents);

        let hzb = [
            device_context.create_texture(hzb_desc, "HZB 0"),
            device_context.create_texture(hzb_desc, "HZB 1"),
        ];

        Self {
            id: RenderSurfaceId::new(),
            window_id,
            extents: render_surface_extents,
            resources: SizeDependentResources::new(
                device_context,
                render_surface_extents,
                pipeline_manager,
            ),
            num_render_frames,
            render_frame_idx: 0,
            presenter_semaphores,
            picking_renderpass: Arc::new(RwLock::new(PickingRenderPass::new(device_context))),
            debug_renderpass: Arc::new(RwLock::new(DebugRenderPass::new(pipeline_manager))),
            egui_renderpass: Arc::new(RwLock::new(EguiPass::new(device_context, pipeline_manager))),
            final_resolve_render_pass: Arc::new(RwLock::new(FinalResolveRenderPass::new(
                device_context,
                pipeline_manager,
            ))),
            presenters: Vec::new(),
            presenting_status: RenderSurfacePresentingStatus::Presenting,
            view_target,
            hzb,
            hzb_cleared: false,
            use_view_target: false,
        }
    }

    pub fn id(&self) -> RenderSurfaceId {
        self.id
    }

    pub fn window_id(&self) -> Option<WindowId> {
        self.window_id
    }

    pub fn extents(&self) -> RenderSurfaceExtents {
        self.extents
    }

    pub fn picking_renderpass(&self) -> Arc<RwLock<PickingRenderPass>> {
        self.picking_renderpass.clone()
    }

    pub fn debug_renderpass(&self) -> Arc<RwLock<DebugRenderPass>> {
        self.debug_renderpass.clone()
    }

    pub fn egui_renderpass(&self) -> Arc<RwLock<EguiPass>> {
        self.egui_renderpass.clone()
    }

    pub fn final_resolve_render_pass(&self) -> Arc<RwLock<FinalResolveRenderPass>> {
        self.final_resolve_render_pass.clone()
    }

    pub fn resize(
        &mut self,
        device_context: &DeviceContext,
        render_surface_extents: RenderSurfaceExtents,
        pipeline_manager: &PipelineManager,
    ) {
        if self.extents != render_surface_extents {
            self.resources = SizeDependentResources::new(
                device_context,
                render_surface_extents,
                pipeline_manager,
            );

            let extents = Extents3D {
                width: render_surface_extents.width(),
                height: render_surface_extents.height(),
                depth: 1,
            };
            let view_desc = TextureDef {
                extents,
                array_length: 1,
                mip_count: 1,
                format: Format::B8G8R8A8_UNORM,
                usage_flags: ResourceUsage::AS_RENDER_TARGET
                    | ResourceUsage::AS_SHADER_RESOURCE
                    | ResourceUsage::AS_UNORDERED_ACCESS
                    | ResourceUsage::AS_TRANSFERABLE,
                resource_flags: ResourceFlags::empty(),
                memory_usage: MemoryUsage::GpuOnly,
                tiling: TextureTiling::Optimal,
            };
            self.view_target = device_context.create_texture(view_desc, "ViewBuffer");

            let hzb_desc = Self::make_hzb_desc(&extents);

            self.hzb = [
                device_context.create_texture(hzb_desc, "HZB 0"),
                device_context.create_texture(hzb_desc, "HZB 1"),
            ];
            self.hzb_cleared = false;

            for presenter in &mut self.presenters {
                presenter.resize(device_context, render_surface_extents);
            }
            self.extents = render_surface_extents;
        }
    }

    pub fn register_presenter<T: 'static + Presenter>(&mut self, create_fn: impl FnOnce() -> T) {
        let presenter = create_fn();
        self.presenters.push(Box::new(presenter));
    }

    pub fn hdr_rt(&self) -> &RenderTarget {
        &self.resources.hdr_rt
    }

    pub fn hdr_rt_mut(&mut self) -> &mut RenderTarget {
        &mut self.resources.hdr_rt
    }

    pub fn depth_rt(&self) -> &RenderTarget {
        &self.resources.depth_rt
    }

    pub fn depth_rt_mut(&mut self) -> &mut RenderTarget {
        &mut self.resources.depth_rt
    }

    pub(crate) fn init_hzb_if_needed(
        &mut self,
        render_context: &mut RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
    ) {
        if !self.resources.hzb_init {
            self.resources
                .depth_rt
                .transition_to(cmd_buffer, ResourceState::DEPTH_WRITE);

            cmd_buffer.cmd_begin_render_pass(
                &[],
                &Some(DepthStencilRenderTargetBinding {
                    texture_view: self.resources.depth_rt.rtv(),
                    depth_load_op: LoadOp::Clear,
                    stencil_load_op: LoadOp::DontCare,
                    depth_store_op: StoreOp::Store,
                    stencil_store_op: StoreOp::DontCare,
                    clear_value: DepthStencilClearValue {
                        depth: 0.0,
                        stencil: 0,
                    },
                }),
            );
            cmd_buffer.cmd_end_render_pass();

            self.generate_hzb(render_context, cmd_buffer);

            self.resources.hzb_init = true;
        }
    }

    pub(crate) fn generate_hzb(
        &mut self,
        render_context: &mut RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
    ) {
        cmd_buffer.with_label("Generate HZB", |cmd_buffer| {
            self.depth_rt_mut()
                .transition_to(cmd_buffer, ResourceState::PIXEL_SHADER_RESOURCE);

            self.get_hzb_surface()
                .generate_hzb(render_context, cmd_buffer, self.depth_rt().srv());

            self.depth_rt_mut()
                .transition_to(cmd_buffer, ResourceState::DEPTH_WRITE);
        });
    }

    pub(crate) fn get_hzb_surface(&self) -> &HzbSurface {
        &self.resources.hzb_surface
    }

    /// Call the `present` method of all the registered presenters.
    /// No op if the render surface is "paused", i.e., it's `presenting`
    /// attribute is `false`.
    pub fn present(&mut self, render_context: &mut RenderContext<'_>) {
        if matches!(
            self.presenting_status,
            RenderSurfacePresentingStatus::Paused
        ) {
            return;
        }

        let mut presenters = std::mem::take(&mut self.presenters);

        for presenter in &mut presenters {
            presenter.as_mut().present(render_context, self);
        }

        self.presenters = presenters;
    }

    //
    // TODO: change that asap. Acquire can't be called more than once per frame.
    // This would result in a crash.
    //
    pub fn acquire(&mut self) -> &Semaphore {
        let render_frame_idx = (self.render_frame_idx + 1) % self.num_render_frames;
        let presenter_sem = &self.presenter_semaphores[render_frame_idx as usize];
        self.render_frame_idx = render_frame_idx;

        presenter_sem
    }

    pub fn presenter_sem(&self) -> &Semaphore {
        &self.presenter_semaphores[self.render_frame_idx as usize]
    }

    pub fn pause(&mut self) -> &mut Self {
        self.presenting_status = RenderSurfacePresentingStatus::Paused;
        self
    }

    pub fn resume(&mut self) -> &mut Self {
        self.presenting_status = RenderSurfacePresentingStatus::Presenting;
        self
    }

    fn make_hzb_desc(extents: &Extents3D) -> TextureDef {
        const SCALE_THRESHOLD: f32 = 0.7;

        let mut hzb_width = 2.0f32.powf((extents.width as f32).log2().floor());
        if hzb_width / extents.width as f32 > SCALE_THRESHOLD {
            hzb_width /= 2.0;
        }
        let mut hzb_height = 2.0f32.powf((extents.height as f32).log2().floor());
        if hzb_height / extents.height as f32 > SCALE_THRESHOLD {
            hzb_height /= 2.0;
        }

        hzb_width = hzb_width.max(4.0);
        hzb_height = hzb_height.max(4.0);

        let mut min_extent = hzb_width.min(hzb_height) as u32;
        let mut mip_count = 1;
        while min_extent != 1 {
            min_extent /= 2;
            mip_count += 1;
        }

        TextureDef {
            extents: Extents3D {
                width: hzb_width as u32,
                height: hzb_height as u32,
                depth: 1,
            },
            array_length: 1,
            mip_count,
            format: Format::R32_SFLOAT,
            usage_flags: ResourceUsage::AS_RENDER_TARGET
                | ResourceUsage::AS_SHADER_RESOURCE
                | ResourceUsage::AS_UNORDERED_ACCESS
                | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        }
    }

    pub fn view_target(&self) -> &Texture {
        &self.view_target
    }

    pub fn use_view_target(&self) -> bool {
        self.use_view_target
    }

    pub fn set_use_view_target(&mut self, enable: bool) {
        self.use_view_target = enable;
    }

    pub(crate) fn hzb(&self) -> [&Texture; 2] {
        [&self.hzb[0], &self.hzb[1]]
    }

    pub(crate) fn clear_hzb(&mut self, cmd_buffer: &mut CommandBuffer) -> bool {
        let mut cleared_this_frame = false;

        if !self.hzb_cleared {
            self.hzb_cleared = true;
            cleared_this_frame = true;

            cmd_buffer.with_label("Clear Prev HZB", |cmd_buffer| {
                for i in 0..2 {
                    for mip in 0..self.hzb[i].definition().mip_count {
                        let hzb_view = self.hzb[i].create_view(TextureViewDef {
                            gpu_view_type: GPUViewType::RenderTarget,
                            view_dimension: ViewDimension::_2D,
                            first_mip: mip,
                            mip_count: 1,
                            plane_slice: PlaneSlice::Default,
                            first_array_slice: 0,
                            array_size: 1,
                        });

                        cmd_buffer.cmd_begin_render_pass(
                            &[ColorRenderTargetBinding {
                                texture_view: &hzb_view,
                                load_op: LoadOp::Clear,
                                store_op: StoreOp::Store,
                                clear_value: ColorClearValue([0.0; 4]),
                            }],
                            &None,
                        );
                        cmd_buffer.cmd_end_render_pass();
                    }
                }
            });
        }

        cleared_this_frame
    }
}
