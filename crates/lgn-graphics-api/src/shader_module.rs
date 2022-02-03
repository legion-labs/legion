use crate::{
    backends::BackendShaderModule, deferred_drop::Drc, DeviceContext, GfxResult, ShaderModuleDef,
};

pub(crate) struct ShaderModuleInner {
    device_context: DeviceContext,
    pub(crate) backend_shader_module: BackendShaderModule,
}

impl Drop for ShaderModuleInner {
    fn drop(&mut self) {
        self.backend_shader_module.destroy(&self.device_context);
    }
}

#[derive(Clone)]
pub struct ShaderModule {
    pub(crate) inner: Drc<ShaderModuleInner>,
}

impl ShaderModule {
    pub fn new(device_context: &DeviceContext, data: ShaderModuleDef<'_>) -> GfxResult<Self> {
        let backend_shader_module = BackendShaderModule::new(device_context, data)?;

        Ok(Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(ShaderModuleInner {
                    device_context: device_context.clone(),
                    backend_shader_module,
                }),
        })
    }
}
