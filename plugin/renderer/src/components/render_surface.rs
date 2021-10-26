use graphics_api::{DefaultApi, DeviceContext, Extents3D, Format, GfxApi, MemoryUsage, ResourceFlags, ResourceUsage, Texture, TextureDef, TextureTiling, TextureViewDef};
use legion_ecs::prelude::Component;
use legion_window::{Window, WindowId};

use crate::{Renderer, TmpRenderPass};

#[derive(Debug, Component)]
pub struct RenderSurface {
    pub window_id : WindowId,
    pub width : u32,
    pub height : u32,
    pub texture : <DefaultApi as GfxApi>::Texture,
    pub texture_rtv : <DefaultApi as GfxApi>::TextureView,
    pub test_renderpass: TmpRenderPass
}

impl RenderSurface {
    pub fn from_window(renderer: &Renderer, window: &Window) -> Self {        

        let device_context = renderer.device_context();
        let texture_def = TextureDef {
            extents: Extents3D {
                width: window.physical_width(),
                height: window.physical_height(),
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R16G16B16A16_SFLOAT,
            usage_flags: ResourceUsage::HAS_RENDER_TARGET_VIEW|ResourceUsage::HAS_SHADER_RESOURCE_VIEW,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };        
        let texture = device_context.create_texture(&texture_def).unwrap();
        let rtv_def = TextureViewDef::as_render_target_view(&texture_def);
        let texture_rtv = texture.create_view(&rtv_def).unwrap();

        Self {
            window_id: window.id(),       
            width: texture_def.extents.width,     
            height: texture_def.extents.height,     
            texture,
            texture_rtv,
            test_renderpass: TmpRenderPass::new(renderer)
        }
    }

    pub fn resize(&mut self, device_context: &<DefaultApi as GfxApi>::DeviceContext, width: u32, height: u32) {
        if (self.width, self.height) != (width, height) {
            
            let mut texture_def = *self.texture.texture_def();            
            texture_def.extents.width = width;
            texture_def.extents.height = height;            
            let texture = device_context.create_texture(&texture_def).unwrap();
            
            let rtv_def = TextureViewDef::as_render_target_view(&texture_def);
            let texture_rtv = texture.create_view(&rtv_def).unwrap();

            self.width = texture_def.extents.width;
            self.height = texture_def.extents.height;
            self.texture = texture;
            self.texture_rtv = texture_rtv;
        }
    }
}

impl Drop for RenderSurface {
    fn drop(&mut self) {
        
    }
}