use super::{VulkanApi, VulkanDeviceContext};
use crate::*;
use std::sync::Arc;

#[derive(Debug)]
struct VulkanShaderInner {
    stage_flags: ShaderStageFlags,
    stages: Vec<ShaderStageDef<VulkanApi>>,
    pipeline_reflection: PipelineReflection,
}

#[derive(Clone, Debug)]
pub struct VulkanShader {
    inner: Arc<VulkanShaderInner>,
}

impl VulkanShader {
    pub fn new(
        _device_context: &VulkanDeviceContext,
        stages: Vec<ShaderStageDef<VulkanApi>>,
    ) -> GfxResult<Self> {
        let pipeline_reflection = PipelineReflection::from_stages(&stages)?;
        let mut stage_flags = ShaderStageFlags::empty();
        for stage in &stages {
            stage_flags |= stage.reflection.shader_stage;
        }

        let inner = VulkanShaderInner {
            stage_flags,
            stages,
            pipeline_reflection,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn stages(&self) -> &[ShaderStageDef<VulkanApi>] {
        &self.inner.stages
    }

    pub fn stage_flags(&self) -> ShaderStageFlags {
        self.inner.stage_flags
    }
}

impl Shader<VulkanApi> for VulkanShader {
    fn pipeline_reflection(&self) -> &PipelineReflection {
        &self.inner.pipeline_reflection
    }
}
