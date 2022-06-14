use std::collections::hash_map::{Values, ValuesMut};
use std::hash::Hash;
use std::{cmp::max, sync::Arc};

use lgn_ecs::prelude::ResMut;
use lgn_graphics_api::{
    CmdCopyTextureParams, ColorClearValue, ColorRenderTargetBinding, CommandBuffer, DeviceContext,
    Extents2D, Extents3D, Format, GPUViewType, LoadOp, MemoryUsage, Offset2D, Offset3D, PlaneSlice,
    ResourceFlags, ResourceState, ResourceUsage, Semaphore, SemaphoreDef, StoreOp, Texture,
    TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef, ViewDimension,
};
use lgn_window::WindowId;
use parking_lot::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

use crate::core::{
    InsertRenderObjectCommand, PrimaryTableCommandBuilder, PrimaryTableView,
    RemoveRenderObjectCommand, RenderObjectId, SecondaryTableHandler, UpdateRenderObjectCommand,
};
use crate::render_pass::PickingRenderPass;
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

pub enum RenderSurfacePresentingStatus {
    Presenting,
    Paused,
}

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ViewportId(Uuid);

impl ViewportId {
    fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Clone)]
pub struct Viewport {
    id: ViewportId,
    offset: Offset2D,
    extents: Extents2D,
    camera_id: Option<RenderObjectId>,
    render_object_id: Option<RenderObjectId>,
}

impl Viewport {
    pub fn new(offset: Offset2D, extents: Extents2D) -> Self {
        Self {
            id: ViewportId::new(),
            offset,
            extents,
            camera_id: None,
            render_object_id: None,
        }
    }

    pub fn resize(&mut self, offset: Offset2D, extents: Extents2D) {
        if self.offset != offset || self.extents != extents {
            self.offset = offset;
            self.extents = extents;
        }
    }

    pub fn offset(&self) -> Offset2D {
        self.offset
    }

    pub fn extents(&self) -> Extents2D {
        self.extents
    }

    pub fn camera_id(&self) -> RenderObjectId {
        self.camera_id.unwrap()
    }

    pub fn set_camera_id(&mut self, camera: RenderObjectId) {
        self.camera_id = Some(camera);
    }

    pub fn render_object_id(&self) -> Option<RenderObjectId> {
        self.render_object_id
    }
}

#[derive(Debug)]
pub struct RenderViewport {
    offset: Offset2D,
    extents: Extents2D,
    camera_id: Option<RenderObjectId>,
}

impl RenderViewport {
    pub fn new(offset: Offset2D, extents: Extents2D, camera: Option<RenderObjectId>) -> Self {
        Self {
            offset,
            extents,
            camera_id: camera,
        }
    }

    pub fn offset(&self) -> Offset2D {
        self.offset
    }

    pub fn extents(&self) -> Extents2D {
        self.extents
    }

    pub fn camera_id(&self) -> Option<RenderObjectId> {
        self.camera_id
    }

    pub fn set_camera_id(&mut self, camera: RenderObjectId) {
        self.camera_id = Some(camera);
    }
}

fn as_render_object(viewport: &Viewport) -> RenderViewport {
    RenderViewport::new(viewport.offset, viewport.extents, viewport.camera_id)
}

pub struct RenderViewportPrivateData {
    view_target: Texture,
    view_target_srv: TextureView,
    hzb: [Texture; 2],
    hzb_cleared: bool,
}

impl RenderViewportPrivateData {
    pub fn new(render_viewport: &RenderViewport, device_context: &DeviceContext) -> Self {
        let extents_3d = render_viewport.extents().to_3d(1);

        let view_desc = TextureDef {
            extents: extents_3d,
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
        let view_target_srv = view_target.create_view(TextureViewDef::as_shader_resource_view(
            view_target.definition(),
        ));

        let hzb_desc = Self::make_hzb_desc(&extents_3d);

        let hzb = [
            device_context.create_texture(hzb_desc, "HZB 0"),
            device_context.create_texture(hzb_desc, "HZB 1"),
        ];

        Self {
            view_target,
            view_target_srv,
            hzb,
            hzb_cleared: false,
        }
    }

    pub fn view_target(&self) -> &Texture {
        &self.view_target
    }

    pub fn view_target_srv(&self) -> &TextureView {
        &self.view_target_srv
    }

    pub(crate) fn hzb(&self) -> [&Texture; 2] {
        [&self.hzb[0], &self.hzb[1]]
    }

    pub fn resize(&mut self, device_context: &DeviceContext, extents: Extents3D) {
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
        self.view_target_srv =
            self.view_target
                .create_view(TextureViewDef::as_shader_resource_view(
                    self.view_target.definition(),
                ));

        let hzb_desc = Self::make_hzb_desc(&extents);

        self.hzb = [
            device_context.create_texture(hzb_desc, "HZB 0"),
            device_context.create_texture(hzb_desc, "HZB 1"),
        ];
        self.hzb_cleared = false;
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

    pub(crate) fn clear_hzb_if_needed(&mut self, cmd_buffer: &mut CommandBuffer) {
        if !self.hzb_cleared {
            self.hzb_cleared = true;

            cmd_buffer.with_label("Clear Prev HZB", |cmd_buffer| {
                for i in 0..2 {
                    for mip in 0..self.hzb[i].definition().mip_count {
                        cmd_buffer.cmd_resource_barrier(
                            &[],
                            &[TextureBarrier::state_transition_for_mip(
                                &self.hzb[i],
                                ResourceState::UNDEFINED,
                                ResourceState::RENDER_TARGET,
                                Some(mip as u8),
                            )],
                        );

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

                        cmd_buffer.cmd_resource_barrier(
                            &[],
                            &[TextureBarrier::state_transition_for_mip(
                                &self.hzb[i],
                                ResourceState::RENDER_TARGET,
                                ResourceState::SHADER_RESOURCE,
                                Some(mip as u8),
                            )],
                        );
                    }
                }
            });
        }
    }
}

pub struct RenderViewportPrivateDataHandler {
    device_context: DeviceContext,
}

impl SecondaryTableHandler<RenderViewport, RenderViewportPrivateData>
    for RenderViewportPrivateDataHandler
{
    fn create(&self, render_viewport: &RenderViewport) -> RenderViewportPrivateData {
        RenderViewportPrivateData::new(render_viewport, &self.device_context)
    }

    fn update(
        &self,
        render_viewport: &RenderViewport,
        render_viewport_private_data: &mut RenderViewportPrivateData,
    ) {
        let viewport_extents = render_viewport.extents.to_3d(1);
        if viewport_extents
            != render_viewport_private_data
                .view_target
                .definition()
                .extents
        {
            render_viewport_private_data.resize(&self.device_context, viewport_extents);
        }
    }

    fn destroy(
        &self,
        _render_viewport: &RenderViewport,
        _render_viewport_private_data: &mut RenderViewportPrivateData,
    ) {
    }
}

impl RenderViewportPrivateDataHandler {
    pub fn new(device_context: DeviceContext) -> Self {
        Self { device_context }
    }
}

pub struct RenderSurface {
    id: RenderSurfaceId,
    window_id: Option<WindowId>,
    extents: RenderSurfaceExtents,
    presenters: Vec<Box<dyn Presenter>>,
    viewports: Vec<Viewport>,
    final_target: Texture,
    final_target_srv: TextureView,

    // tmp
    num_render_frames: u64,
    render_frame_idx: u64,
    presenter_semaphores: Vec<Semaphore>,
    picking_renderpass: Arc<RwLock<PickingRenderPass>>,
    presenting_status: RenderSurfacePresentingStatus,
}

impl RenderSurface {
    pub fn new(
        window_id: WindowId,
        renderer: &Renderer,
        render_surface_extents: RenderSurfaceExtents,
    ) -> Self {
        Self::new_internal(Some(window_id), renderer, render_surface_extents)
    }

    pub fn new_offscreen_window(
        renderer: &Renderer,
        render_surface_extents: RenderSurfaceExtents,
    ) -> Self {
        Self::new_internal(None, renderer, render_surface_extents)
    }

    fn new_internal(
        window_id: Option<WindowId>,
        renderer: &Renderer,
        render_surface_extents: RenderSurfaceExtents,
    ) -> Self {
        let num_render_frames = renderer.num_render_frames();
        let device_context = renderer.device_context();
        let presenter_semaphores = (0..num_render_frames)
            .map(|_| device_context.create_semaphore(SemaphoreDef::default()))
            .collect();

        let final_target_desc = TextureDef {
            extents: Extents3D {
                width: render_surface_extents.width(),
                height: render_surface_extents.height(),
                depth: 1,
            },
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
        let final_target = device_context.create_texture(final_target_desc, "FinalTarget");
        let final_target_srv = final_target.create_view(TextureViewDef::as_shader_resource_view(
            final_target.definition(),
        ));

        Self {
            id: RenderSurfaceId::new(),
            window_id,
            extents: render_surface_extents,
            num_render_frames,
            render_frame_idx: 0,
            presenter_semaphores,
            final_target,
            final_target_srv,
            picking_renderpass: Arc::new(RwLock::new(PickingRenderPass::new(device_context))),
            presenters: Vec::new(),
            presenting_status: RenderSurfacePresentingStatus::Presenting,
            viewports: vec![],
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

    pub fn viewports(&self) -> &Vec<Viewport> {
        &self.viewports
    }

    pub fn viewports_mut(&mut self) -> &mut Vec<Viewport> {
        &mut self.viewports
    }

    pub fn add_viewport(&mut self, viewport: Viewport) {
        self.viewports.push(viewport);
    }

    pub fn final_target(&self) -> &Texture {
        &self.final_target
    }

    pub fn final_target_srv(&self) -> &TextureView {
        &self.final_target_srv
    }

    pub fn resize(
        &mut self,
        device_context: &DeviceContext,
        render_surface_extents: RenderSurfaceExtents,
    ) {
        if self.extents != render_surface_extents {
            let extents = Extents3D {
                width: render_surface_extents.width(),
                height: render_surface_extents.height(),
                depth: 1,
            };

            let final_target_desc = TextureDef {
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
            self.final_target = device_context.create_texture(final_target_desc, "FinalTarget");
            self.final_target_srv =
                self.final_target
                    .create_view(TextureViewDef::as_shader_resource_view(
                        self.final_target.definition(),
                    ));

            for presenter in &mut self.presenters {
                presenter.resize(device_context, render_surface_extents);
            }

            // TODO(jsg): is this what we want to do on surface resize? Resize all viewports proportionally to how much of the surface they used to cover?
            for viewport in &mut self.viewports {
                let viewport_relative_position = (
                    viewport.offset.x as f32 / self.extents.width() as f32,
                    viewport.offset.y as f32 / self.extents.height() as f32,
                );

                let viewport_relative_size = (
                    viewport.extents.width as f32 / self.extents.width() as f32,
                    viewport.extents.height as f32 / self.extents.height() as f32,
                );

                let viewport_new_offset = Offset2D {
                    x: (extents.width as f32 * viewport_relative_position.0) as i32,
                    y: (extents.height as f32 * viewport_relative_position.1) as i32,
                };

                let viewport_new_extents = Extents2D {
                    width: (extents.width as f32 * viewport_relative_size.0) as u32,
                    height: (extents.height as f32 * viewport_relative_size.1) as u32,
                };

                viewport.resize(viewport_new_offset, viewport_new_extents);
            }

            self.extents = render_surface_extents;
        }
    }

    pub fn composite_viewports(
        &self,
        render_viewports: &[&RenderViewport],
        render_viewports_private_data: &[&RenderViewportPrivateData],
        cmd_buffer: &mut CommandBuffer,
    ) {
        for (i, render_viewport_private_data) in render_viewports_private_data.iter().enumerate() {
            let render_viewport = render_viewports[i];

            let view_target = render_viewport_private_data.view_target();

            cmd_buffer.cmd_resource_barrier(
                &[],
                &[
                    TextureBarrier::state_transition(
                        &self.final_target,
                        ResourceState::SHADER_RESOURCE,
                        ResourceState::COPY_DST,
                    ),
                    TextureBarrier::state_transition(
                        view_target,
                        ResourceState::RENDER_TARGET,
                        ResourceState::COPY_SRC,
                    ),
                ],
            );

            cmd_buffer.cmd_copy_image(
                view_target,
                &self.final_target,
                &CmdCopyTextureParams {
                    src_state: ResourceState::COPY_SRC,
                    dst_state: ResourceState::COPY_DST,
                    src_offset: Offset3D { x: 0, y: 0, z: 0 },
                    dst_offset: render_viewport.offset().to_3d(0),
                    src_mip_level: 0,
                    dst_mip_level: 0,
                    src_array_slice: 0,
                    dst_array_slice: 0,
                    src_plane_slice: PlaneSlice::Default,
                    dst_plane_slice: PlaneSlice::Default,
                    extent: view_target.definition().extents,
                },
            );

            cmd_buffer.cmd_resource_barrier(
                &[],
                &[
                    TextureBarrier::state_transition(
                        &self.final_target,
                        ResourceState::COPY_DST,
                        ResourceState::SHADER_RESOURCE,
                    ),
                    TextureBarrier::state_transition(
                        view_target,
                        ResourceState::COPY_SRC,
                        ResourceState::RENDER_TARGET,
                    ),
                ],
            );
        }
    }

    pub fn register_presenter<T: 'static + Presenter>(&mut self, create_fn: impl FnOnce() -> T) {
        let presenter = create_fn();
        self.presenters.push(Box::new(presenter));
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
}

// We need to manually find out which viewports have been added, modified or removed.
struct ChangeLog<T> {
    pub added: Vec<T>,
    pub modified: Vec<T>,
    pub removed: Vec<T>,
}

pub(crate) struct EcsToRenderViewport {
    view: PrimaryTableView<RenderViewport>,
    prev_viewport_ids: Vec<ViewportId>,
    prev_viewports: Vec<Viewport>, // clones of the viewports -- do not try to modify
}

impl EcsToRenderViewport {
    pub fn new(view: PrimaryTableView<RenderViewport>) -> Self {
        Self {
            view,
            prev_viewport_ids: vec![],
            prev_viewports: vec![],
        }
    }

    pub fn alloc_id(&self) -> RenderObjectId {
        self.view.allocate()
    }

    pub fn command_builder(&self) -> PrimaryTableCommandBuilder {
        self.view.command_builder()
    }

    fn compare_viewports(
        &mut self,
        new_viewport_ids: &[ViewportId],
        viewports: &[&mut Viewport],
    ) -> ChangeLog<ViewportId> {
        let mut added = new_viewport_ids.to_vec();
        added.retain(|v_id| {
            let mut found = false;
            for v2_id in &self.prev_viewport_ids {
                if v_id == v2_id {
                    found = true;
                    break;
                }
            }
            !found
        });

        let mut removed = self.prev_viewport_ids.clone();
        removed.retain(|v_id| {
            let mut found = false;
            for v2_id in new_viewport_ids {
                if v_id == v2_id {
                    found = true;
                    break;
                }
            }
            !found
        });

        let mut modified = new_viewport_ids.to_vec();
        modified.retain(|v_id| {
            let mut changed = false;
            for v2_id in &self.prev_viewport_ids {
                if v_id == v2_id {
                    let v = viewports.iter().find(|v| v.id == *v_id).unwrap();
                    let v2 = self
                        .prev_viewports
                        .iter()
                        .find(|v2| v2.id == *v2_id)
                        .unwrap();
                    changed = v.offset != v2.offset
                        || v.extents != v2.extents
                        || v.camera_id != v2.camera_id;
                    break;
                }
            }
            changed
        });

        ChangeLog::<ViewportId> {
            added,
            modified,
            removed,
        }
    }

    fn update(&mut self, new_viewport_ids: &[ViewportId], viewports: &[&mut Viewport]) {
        self.prev_viewport_ids = new_viewport_ids.to_vec();

        let mut viewports_clone: Vec<Viewport> = vec![];
        for viewport in viewports {
            let viewport = (**viewport).clone();
            viewports_clone.push(viewport);
        }
        self.prev_viewports = viewports_clone;
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub(crate) fn reflect_viewports(
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
    mut ecs_to_render: ResMut<'_, EcsToRenderViewport>,
) {
    let mut render_commands = ecs_to_render.command_builder();

    let mut all_viewports = vec![];
    for render_surface in render_surfaces.iter_mut() {
        let viewports = render_surface.viewports_mut();
        for viewport in viewports {
            all_viewports.push(viewport);
        }
    }

    let viewport_ids: Vec<ViewportId> = all_viewports.iter().map(|v| v.id).collect();

    let changes = ecs_to_render.compare_viewports(&viewport_ids, &all_viewports);

    for v_id in changes.removed {
        if let Some(viewport) = ecs_to_render.prev_viewports.iter().find(|v| v.id == v_id) {
            render_commands.push(RemoveRenderObjectCommand {
                render_object_id: viewport.render_object_id.unwrap(),
            });
        } else {
            panic!();
        }
    }

    ecs_to_render.update(&viewport_ids, &all_viewports);

    for v_id in changes.added {
        let render_object_id = ecs_to_render.alloc_id();
        if let Some(viewport) = all_viewports.iter_mut().find(|v| v.id == v_id) {
            viewport.render_object_id = Some(render_object_id);
            render_commands.push(InsertRenderObjectCommand::<RenderViewport> {
                render_object_id,
                data: as_render_object(viewport),
            });
        } else {
            panic!();
        }
    }

    for v_id in changes.modified {
        if let Some(viewport) = all_viewports.iter().find(|v| v.id == v_id) {
            if let Some(render_object_id) = viewport.render_object_id {
                render_commands.push(UpdateRenderObjectCommand::<RenderViewport> {
                    render_object_id,
                    data: as_render_object(viewport),
                });
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
}
