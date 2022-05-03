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
    pub fn new(
        device_context: &DeviceContext,
        name: &str,
        extents: RenderSurfaceExtents,
        format: Format,
        usage_flags: ResourceUsage,
        view_type: GPUViewType,
    ) -> Self {
        let texture = device_context.create_texture(TextureDef {
				name: name.to_string(),
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
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        });

        let srv = texture.create_view(TextureViewDef::as_shader_resource_view(
            texture.definition(),
        ));

        let rtv = texture.create_view(TextureViewDef::as_render_view(
            texture.definition(),
            view_type,
        ));

        Self {
            texture,
            srv,
            rtv,
            state: ResourceState::UNDEFINED,
        }
    }

    pub fn transition_to(
        &mut self,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        dst_state: ResourceState,
    ) {
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

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn srv(&self) -> &TextureView {
        &self.srv
    }

    pub fn rtv(&self) -> &TextureView {
        &self.rtv
    }
}
