use graphics_api::{
    CommandBuffer, DefaultApi, DeviceContext, Extents3D, Format, GfxApi, MemoryUsage,
    ResourceFlags, ResourceState, ResourceUsage, Texture, TextureBarrier, TextureDef,
    TextureTiling, TextureViewDef,
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

#[derive(Debug, Component)]
pub struct RenderSurface {
    pub id: RenderSurfaceId,
    pub width: u32,
    pub height: u32,
    pub test_renderpass: TmpRenderPass,
    texture: <DefaultApi as GfxApi>::Texture,
    texture_srv: <DefaultApi as GfxApi>::TextureView,
    texture_rtv: <DefaultApi as GfxApi>::TextureView,
    texture_state: ResourceState,
}

impl RenderSurface {
    pub fn new(renderer: &Renderer, width: u32, height: u32) -> Self {
        let device_context = renderer.device_context();
        let texture_def = TextureDef {
            extents: Extents3D {
                width,
                height,
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

        Self {
            id: RenderSurfaceId::new(),
            width: texture_def.extents.width,
            height: texture_def.extents.height,
            texture,
            texture_srv,
            texture_rtv,
            texture_state: ResourceState::UNDEFINED,
            test_renderpass: TmpRenderPass::new(renderer),
        }
    }

    pub fn resize(&mut self, renderer: &Renderer, width: u32, height: u32) {
        if (self.width, self.height) != (width, height) {
            let device_context = renderer.device_context();

            let mut texture_def = *self.texture.texture_def();
            texture_def.extents.width = width;
            texture_def.extents.height = height;
            let texture = device_context.create_texture(&texture_def).unwrap();

            let srv_def = TextureViewDef::as_shader_resource_view(&texture_def);
            let texture_srv = texture.create_view(&srv_def).unwrap();

            let rtv_def = TextureViewDef::as_render_target_view(&texture_def);
            let texture_rtv = texture.create_view(&rtv_def).unwrap();

            self.width = texture_def.extents.width;
            self.height = texture_def.extents.height;
            self.texture = texture;
            self.texture_srv = texture_srv;
            self.texture_rtv = texture_rtv;
            self.texture_state = ResourceState::UNDEFINED;
        }
    }

    pub fn texture(&self) -> &<DefaultApi as GfxApi>::Texture {
        &self.texture
    }

    pub fn render_target_view(&self) -> &<DefaultApi as GfxApi>::TextureView {
        &self.texture_rtv
    }

    pub fn shader_resource_view(&self) -> &<DefaultApi as GfxApi>::TextureView {
        &self.texture_srv
    }

    pub fn transition_to(
        &mut self,
        cmd_buffer: &<DefaultApi as GfxApi>::CommandBuffer,
        dst_state: ResourceState,
    ) {
        let src_state = self.texture_state;
        let dst_state = dst_state;

        if src_state != dst_state {
            cmd_buffer
                .cmd_resource_barrier(
                    &[],
                    &[TextureBarrier::<DefaultApi>::state_transition(
                        &self.texture,
                        src_state,
                        dst_state,
                    )],
                )
                .unwrap();
            self.texture_state = dst_state;
        }
    }
}

impl Drop for RenderSurface {
    fn drop(&mut self) {}
}
