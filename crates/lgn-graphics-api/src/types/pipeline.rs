#![allow(clippy::too_many_lines)]

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanPipeline;
use crate::DeviceContext;
use crate::{
    deferred_drop::Drc, ComputePipelineDef, GfxResult, GraphicsPipelineDef, PipelineType,
    RootSignature,
};

pub(crate) struct PipelineInner {
    root_signature: RootSignature,
    pipeline_type: PipelineType,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_pipeline: VulkanPipeline,
}

impl Drop for PipelineInner {
    fn drop(&mut self) {
        #[cfg(any(feature = "vulkan"))]
        self.platform_pipeline
            .destroy(self.root_signature.device_context());
    }
}

#[derive(Clone)]
pub struct Pipeline {
    pub(crate) inner: Drc<PipelineInner>,
}

impl Pipeline {
    pub fn new_graphics_pipeline(
        device_context: &DeviceContext,
        pipeline_def: &GraphicsPipelineDef<'_>,
    ) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_pipeline = VulkanPipeline::new_graphics_pipeline(device_context, pipeline_def)
            .map_err(|e| {
                lgn_tracing::error!("Error creating graphics pipeline {:?}", e);
                ash::vk::Result::ERROR_UNKNOWN
            })?;

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(PipelineInner {
                pipeline_type: PipelineType::Graphics,
                root_signature: pipeline_def.root_signature.clone(),
                #[cfg(any(feature = "vulkan"))]
                platform_pipeline,
            }),
        })
    }

    pub fn new_compute_pipeline(
        device_context: &DeviceContext,
        pipeline_def: &ComputePipelineDef<'_>,
    ) -> GfxResult<Self> {
        #[cfg(feature = "vulkan")]
        let platform_pipeline = VulkanPipeline::new_compute_pipeline(device_context, pipeline_def)
            .map_err(|e| {
                lgn_tracing::error!("Error creating compute pipeline {:?}", e);
                ash::vk::Result::ERROR_UNKNOWN
            })?;

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(PipelineInner {
                pipeline_type: PipelineType::Compute,
                root_signature: pipeline_def.root_signature.clone(),
                #[cfg(any(feature = "vulkan"))]
                platform_pipeline,
            }),
        })
    }

    pub fn pipeline_type(&self) -> PipelineType {
        self.inner.pipeline_type
    }

    pub fn root_signature(&self) -> &RootSignature {
        &self.inner.root_signature
    }
}
