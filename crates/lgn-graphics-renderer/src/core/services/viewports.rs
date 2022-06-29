use lgn_graphics_api::{
    ColorClearValue, ColorRenderTargetBinding, CommandBuffer, DeviceContext, Extents2D, Extents3D,
    Format, GPUViewType, LoadOp, MemoryUsage, Offset2D, PlaneSlice, ResourceFlags, ResourceState,
    ResourceUsage, StoreOp, Texture, TextureBarrier, TextureDef, TextureTiling, TextureView,
    TextureViewDef, ViewDimension,
};
use uuid::Uuid;

use crate::core::{RenderObjectId, RenderResources, SecondaryTableHandler};

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

    pub fn id(&self) -> ViewportId {
        self.id
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

    pub fn render_object_id(&self) -> Option<RenderObjectId> {
        self.render_object_id
    }

    pub fn set_render_object_id(&mut self, render_object_id: RenderObjectId) {
        self.render_object_id = Some(render_object_id);
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

pub fn as_render_object(viewport: &Viewport) -> RenderViewport {
    RenderViewport::new(viewport.offset, viewport.extents, viewport.camera_id)
}

pub struct RenderViewportRendererData {
    view_target: Texture,
    view_target_srv: TextureView,
    hzb: [Texture; 2],
    hzb_cleared: bool,
}

impl RenderViewportRendererData {
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

impl SecondaryTableHandler<RenderViewport, RenderViewportRendererData>
    for RenderViewportPrivateDataHandler
{
    fn insert(
        &self,
        _render_resources: &RenderResources,
        _render_object_id: RenderObjectId,
        render_viewport: &RenderViewport,
    ) -> RenderViewportRendererData {
        RenderViewportRendererData::new(render_viewport, &self.device_context)
    }

    fn update(
        &self,
        _render_resources: &RenderResources,
        _render_object_id: RenderObjectId,
        render_viewport: &RenderViewport,
        render_viewport_private_data: &mut RenderViewportRendererData,
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

    fn remove(
        &self,
        _render_resources: &RenderResources,
        _render_object_id: RenderObjectId,
        _render_viewport: &RenderViewport,
        _render_viewport_private_data: &mut RenderViewportRendererData,
    ) {
    }
}

impl RenderViewportPrivateDataHandler {
    pub fn new(device_context: DeviceContext) -> Self {
        Self { device_context }
    }
}
