use super::{VulkanApi, VulkanDeviceContext};
use crate::backends::deferred_drop::Drc;
use crate::{GfxResult, PipelineReflection, Shader, ShaderStageDef, ShaderStageFlags};

#[derive(Debug)]
struct VulkanShaderInner {
    stage_flags: ShaderStageFlags,
    stages: Vec<ShaderStageDef<VulkanApi>>,
    pipeline_reflection: PipelineReflection,
}

#[derive(Clone, Debug)]
pub struct VulkanShader {
    inner: Drc<VulkanShaderInner>,
}

impl VulkanShader {
    pub fn new(
        device_context: &VulkanDeviceContext,
        stages: Vec<ShaderStageDef<VulkanApi>>,
        pipeline_reflection: &PipelineReflection,
    ) -> GfxResult<Self> {
        // let pipeline_reflection = PipelineReflection::from_stages(&stages)?;
        let mut stage_flags = ShaderStageFlags::empty();
        for stage in &stages {
            // stage_flags |= stage.reflection.shader_stage;
            stage_flags |= stage.shader_stage
        }

        let inner = VulkanShaderInner {
            stage_flags,
            stages,
            pipeline_reflection: pipeline_reflection.clone(),
        };

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(inner),
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
