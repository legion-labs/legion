use std::sync::{Arc, Mutex};

use lgn_ecs::prelude::{Changed, Query};
use lgn_graphics_api::{
    DeviceContext, Extents3D, Format, MemoryUsage, ResourceFlags, ResourceUsage, Texture,
    TextureDef, TextureTiling, TextureView, TextureViewDef,
};

use crate::{
    components::{upload_texture_data, TextureComponent},
    hl_gfx_api::HLCommandBuffer,
};

pub struct BindlessTextureManagerInner {
    default_black_texture: Texture,
    uploaded_textures: Vec<Texture>,
    descriptor_array: Vec<TextureView>,
    default_black_texture_uploaded: bool,
}
pub struct BindlessTextureManager {
    inner: Arc<Mutex<BindlessTextureManagerInner>>,
}

impl BindlessTextureManager {
    pub fn new(device_context: &DeviceContext, array_size: usize) -> Self {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 4,
                height: 4,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let default_black_texture = device_context.create_texture(&texture_def);

        let default_black_texture_view_def = TextureViewDef::as_shader_resource_view(&texture_def);

        let descriptor_array =
            vec![default_black_texture.create_view(&default_black_texture_view_def,); array_size];

        Self {
            inner: Arc::new(Mutex::new(BindlessTextureManagerInner {
                default_black_texture,
                uploaded_textures: Vec::new(),
                descriptor_array,
                default_black_texture_uploaded: false,
            })),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn update_textures(
        &self,
        device_context: &DeviceContext,
        cmd_buffer: &HLCommandBuffer<'_>,
        mut updated_texture_components: Query<
            '_,
            '_,
            &mut TextureComponent,
            Changed<TextureComponent>,
        >,
    ) {
        let mut inner = self.inner.lock().unwrap();

        if !inner.default_black_texture_uploaded {
            let mut texture_data = [0u8; 64];
            for index in 0..16 {
                texture_data[index * 4] = 1;
            }
            upload_texture_data(
                device_context,
                cmd_buffer,
                &inner.default_black_texture,
                &texture_data,
                0,
            );
            inner.default_black_texture_uploaded = true;
        }

        for mut texture_component in updated_texture_components.iter_mut() {
            let new_texture = device_context.create_texture(texture_component.texture_def());
            let texture_view_def =
                TextureViewDef::as_shader_resource_view(texture_component.texture_def());

            texture_component.upload_texture(device_context, cmd_buffer, &new_texture);

            inner.descriptor_array[texture_component.texture_id() as usize] =
                new_texture.create_view(&texture_view_def);
            inner.uploaded_textures.push(new_texture);
        }
    }

    pub fn default_black_texture_view(&self) -> TextureView {
        let inner = self.inner.lock().unwrap();

        inner.descriptor_array[0].clone()
    }

    pub fn bindless_texures_for_update(&self) -> Vec<TextureView> {
        let inner = self.inner.lock().unwrap();

        inner.descriptor_array.clone()
    }
}
