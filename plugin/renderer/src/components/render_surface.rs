use std::cmp::max;

use graphics_api::{
    CommandBuffer, Extents2D, Extents3D, Format, MemoryUsage, ResourceFlags, ResourceState,
    ResourceUsage, Texture, TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef,
};
use legion_ecs::prelude::Component;
use legion_utils::Uuid;

use crate::{Renderer, TmpRenderPass};

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RenderSurfaceId(Uuid);

impl RenderSurfaceId {
    fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, PartialEq)]
pub struct RenderSurfaceExtents {
    extents_2d: Extents2D,
}

impl RenderSurfaceExtents {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            extents_2d: Extents2D {
                width: max(1u32, width),
                height: max(1u32, height),
            },
        }
    }

    pub fn width(&self) -> u32 {
        self.extents_2d.width
    }

    pub fn height(&self) -> u32 {
        self.extents_2d.height
    }
}

#[derive(Component)]
pub struct RenderSurface {
    id: RenderSurfaceId,
    extents: RenderSurfaceExtents,
    texture: Texture,
    texture_srv: TextureView,
    texture_rtv: TextureView,
    texture_state: ResourceState,

    depth_texture: Texture,
    depth_texture_view: TextureView,

    // tmp
    pub test_renderpass: TmpRenderPass,
}

impl RenderSurface {
    pub fn new(renderer: &Renderer, extents: RenderSurfaceExtents) -> Self {
        Self::new_with_id(RenderSurfaceId::new(), renderer, extents)
    }

    pub fn resize(&mut self, renderer: &Renderer, extents: RenderSurfaceExtents) {
        if (self.extents) != extents {
            *self = Self::new_with_id(self.id, renderer, extents);
        }
    }

    pub fn id(&self) -> RenderSurfaceId {
        self.id
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn render_target_view(&self) -> &TextureView {
        &self.texture_rtv
    }

    pub fn shader_resource_view(&self) -> &TextureView {
        &self.texture_srv
    }

    pub fn depth_texture_view(&self) -> &TextureView {
        &self.depth_texture_view
    }

    pub fn transition_to(&mut self, cmd_buffer: &CommandBuffer, dst_state: ResourceState) {
        let src_state = self.texture_state;
        let dst_state = dst_state;

        if src_state != dst_state {
            cmd_buffer
                .cmd_resource_barrier(
                    &[],
                    &[TextureBarrier::state_transition(
                        &self.texture,
                        src_state,
                        dst_state,
                    )],
                )
                .unwrap();
            self.texture_state = dst_state;
        }
    }

    fn new_with_id(
        id: RenderSurfaceId,
        renderer: &Renderer,
        extents: RenderSurfaceExtents,
    ) -> Self {
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

        let depth_def = TextureDef {
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

        let depth_texture = device_context.create_texture(&depth_def).unwrap();
        let depth_texture_view_def = TextureViewDef::as_depth_stencil_view(&depth_def);
        let depth_texture_view = depth_texture.create_view(&depth_texture_view_def).unwrap();

        Self {
            id,
            extents,
            texture,
            texture_srv,
            texture_rtv,
            texture_state: ResourceState::UNDEFINED,
            depth_texture,
            depth_texture_view,
            test_renderpass: TmpRenderPass::new(renderer),
        }
    }
}

impl Drop for RenderSurface {
    fn drop(&mut self) {}
}
