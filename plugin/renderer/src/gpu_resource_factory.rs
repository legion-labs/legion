// use graphics_api::{DefaultApi, DeviceContext, GfxApi, SwapchainDef, TextureDef};

// pub struct GPUResourceFactory {
//     device_context: <DefaultApi as GfxApi>::DeviceContext
// }

// impl GPUResourceFactory {
//     pub(crate) fn new(device_context: <DefaultApi as GfxApi>::DeviceContext) -> Self {
//         GPUResourceFactory{
//             device_context
//         }
//     }

//     pub fn create_swapchain(
//         &self, 
//         raw_window_handle: &dyn raw_window_handle::HasRawWindowHandle,
//         swapchain_def: &SwapchainDef
//     ) -> <DefaultApi as GfxApi>::Swapchain {
//         self.device_context.create_swapchain(raw_window_handle, swapchain_def).unwrap()
//     }

//     pub fn create_texture(&self, texture_def: &TextureDef) -> <DefaultApi as GfxApi>::Texture {
//         self.device_context.create_texture(texture_def).unwrap()
//     }

//     pub fn device_context(&self) -> &<DefaultApi as GfxApi>::DeviceContext {
//         &self.device_context
//     }
// }