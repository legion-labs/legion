use graphics_api::{DefaultApi, DeviceContext, GfxApi, TextureDef};

pub struct GPUResourceFactory {
    device_context: <DefaultApi as GfxApi>::DeviceContext
}

impl GPUResourceFactory {
    pub(crate) fn new(device_context: <DefaultApi as GfxApi>::DeviceContext) -> Self {
        GPUResourceFactory{
            device_context
        }
    }

    pub fn create_texture(&self, texture_def: &TextureDef) -> <DefaultApi as GfxApi>::Texture {
        self.device_context.create_texture(texture_def).unwrap()
    }
}