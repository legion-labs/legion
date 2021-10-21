use graphics_api::{DefaultApi, Extents3D, Format, GfxApi, MemoryUsage, ResourceFlags, ResourceUsage, Texture, TextureDef, TextureTiling};
use legion_ecs::prelude::Component;
use legion_window::{Window, WindowId};

use crate::{GPUResourceFactory};

#[derive(Debug, Component)]
pub struct RenderSurface {
    pub id : WindowId,
    pub width : u32,
    pub height : u32,
    pub texture : <DefaultApi as GfxApi>::Texture,
}

impl RenderSurface {
    pub fn from_window(gpu_resource_factory: &GPUResourceFactory, window: &Window) -> Self {        

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
        let texture = gpu_resource_factory.create_texture(&texture_def);

        Self {
            id: window.id(),       
            width: texture_def.extents.width,     
            height: texture_def.extents.height,     
            texture
        }
    }

    pub fn resize(&mut self, gpu_resource_factory: &GPUResourceFactory, width: u32, height: u32) {
        if (self.width, self.height) != (width, height) {
            let mut texture_def = *self.texture.texture_def();            
            texture_def.extents.width = width;
            texture_def.extents.height = height;
            let texture = gpu_resource_factory.create_texture(&texture_def);
            self.width = texture_def.extents.width;
            self.height = texture_def.extents.height;
            self.texture = texture;
        }
    }
}