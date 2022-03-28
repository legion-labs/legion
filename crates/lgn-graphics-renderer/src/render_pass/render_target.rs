use lgn_graphics_api::{
    DeviceContext, Extents3D, Format, GPUViewType, MemoryUsage, ResourceFlags, ResourceState,
    ResourceUsage, Texture, TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef,
};

use crate::{components::RenderSurfaceExtents, hl_gfx_api::HLCommandBuffer};

pub struct RenderTarget {
    texture: Texture,
    srv: TextureView,
    rtv: TextureView,
    state: ResourceState,
}

impl RenderTarget {
    pub(crate) fn new(
        device_context: &DeviceContext,
        extents: RenderSurfaceExtents,
        format: Format,
        usage_flags: ResourceUsage,
        view_type: GPUViewType,
    ) -> Self {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: extents.width(),
                height: extents.height(),
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format,
            usage_flags,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };
        let texture = device_context.create_texture(&texture_def);

        let srv_def = TextureViewDef::as_shader_resource_view(&texture_def);
        let srv = texture.create_view(&srv_def);

        let rtv_def = TextureViewDef::as_render_view(&texture_def, view_type);
        let rtv = texture.create_view(&rtv_def);

        Self {
            texture,
            srv,
            rtv,
            state: ResourceState::UNDEFINED,
        }
    }

    pub fn transition_to(&mut self, cmd_buffer: &HLCommandBuffer<'_>, dst_state: ResourceState) {
        let src_state = self.state;
        let dst_state = dst_state;

        if src_state != dst_state {
            cmd_buffer.resource_barrier(
                &[],
                &[TextureBarrier::state_transition(
                    &self.texture,
                    src_state,
                    dst_state,
                )],
            );
            self.state = dst_state;
        }
    }

    pub(crate) fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn srv(&self) -> &TextureView {
        &self.srv
    }

    pub(crate) fn rtv(&self) -> &TextureView {
        &self.rtv
    }
}
