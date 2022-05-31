use crate::backends::BackendRootSignature;
use crate::deferred_drop::Drc;
use crate::{DescriptorSetLayout, DeviceContext};

// Not currently exposed
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct DynamicDescriptorIndex(pub(crate) u32);
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct PushConstantIndex(pub(crate) u32);

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct PushConstantDef {
    pub size: u32,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RootSignatureDef {
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
    pub push_constant_def: Option<PushConstantDef>,
}

#[derive(Debug)]
pub(crate) struct RootSignatureInner {
    device_context: DeviceContext,
    definition: RootSignatureDef,
    pub(crate) backend_root_signature: BackendRootSignature,
}

impl PartialEq for RootSignatureInner {
    fn eq(&self, other: &Self) -> bool {
        self.definition == other.definition
            && self.backend_root_signature == other.backend_root_signature
    }
}

impl Drop for RootSignatureInner {
    fn drop(&mut self) {
        self.backend_root_signature.destroy(&self.device_context);
    }
}

#[derive(Debug, Clone)]
pub struct RootSignature {
    pub(crate) inner: Drc<RootSignatureInner>,
}

impl PartialEq for RootSignature {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl RootSignature {
    pub fn new(device_context: &DeviceContext, definition: RootSignatureDef) -> Self {
        let backend_root_signature = BackendRootSignature::new(device_context, definition.clone());

        let inner = RootSignatureInner {
            device_context: device_context.clone(),
            definition,
            backend_root_signature,
        };

        Self {
            inner: device_context.deferred_dropper().new_drc(inner),
        }
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    pub fn definition(&self) -> &RootSignatureDef {
        &self.inner.definition
    }
}
