use crate::types::ShaderStageFlags;
use crate::{GfxResult, ShaderResourceType, MAX_DESCRIPTOR_SET_LAYOUTS};

#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};

/// Indicates where a resource is bound
#[derive(PartialEq, Eq, Hash, Default)]
pub struct ShaderResourceBindingKey {
    pub set: u32,
    pub binding: u32,
}

/// A data source within a shader. Often a descriptor or push constant.
///
/// A `ShaderResource` may be specified by hand or generated using shader-compiler
//TODO: Consider separate type for bindings vs. push constants
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub struct ShaderResource {
    pub name: String,
    pub shader_resource_type: ShaderResourceType,
    pub binding: u32,
    pub set_index: u32,
    pub element_count: u32,
    pub used_in_shader_stages: ShaderStageFlags,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PushConstant {
    pub used_in_shader_stages: ShaderStageFlags,
    pub size: u32,
}

impl ShaderResource {
    pub fn element_count_normalized(&self) -> u32 {
        // Assume 0 = default of 1
        self.element_count.max(1)
    }

    pub fn validate(&self) -> GfxResult<()> {
        if self.set_index as usize >= MAX_DESCRIPTOR_SET_LAYOUTS {
            return Err(format!(
                "Descriptor (set={:?} binding={:?}) named {:?} has a set index >= 4. This is not supported",
                self.set_index, self.binding, self.name,
            ).into());
        }

        Ok(())
    }
}

/// Reflection data for a pipeline, created by merging shader stage reflection data
#[derive(Clone, Debug)]
pub struct PipelineReflection {
    pub shader_resources: Vec<ShaderResource>,
    pub push_constant: Option<PushConstant>,
    pub compute_threads_per_group: Option<[u32; 3]>,
}

impl Default for PipelineReflection {
    fn default() -> Self {
        Self {
            shader_resources: Vec::new(),
            push_constant: None,
            compute_threads_per_group: None,
        }
    }
}

impl PipelineReflection {
    pub fn merge(left_op: &Self, right_op: &Self) -> GfxResult<Self> {
        let arr = [left_op, right_op];
        Ok(Self {
            shader_resources: merge_resources(&arr).unwrap(),
            push_constant: merge_pushconstant(&arr).unwrap(),
            compute_threads_per_group: None,
        })
    }
}

fn merge_pushconstant(reflections: &[&PipelineReflection]) -> GfxResult<Option<PushConstant>> {
    let mut result: Option<PushConstant> = None;

    for reflection in reflections {
        if let Some(push_constant) = &mut result {
            if let Some(other_push_constant) = reflection.push_constant {
                if push_constant.size != other_push_constant.size {
                    let message = "Cannot merge pushconstants of different size".to_owned();
                    log::error!("{}", message);
                    return Err(message.into());
                }
                push_constant.used_in_shader_stages |= other_push_constant.used_in_shader_stages;
            }
        } else {
            result = reflection.push_constant;
        }
    }

    Ok(result)
}

fn merge_resources(reflections: &[&PipelineReflection]) -> GfxResult<Vec<ShaderResource>> {
    let mut result = Vec::<ShaderResource>::new();

    for reflection in reflections {
        if !result.is_empty() {
            for other_shader_resource in &reflection.shader_resources {
                let found = result
                    .iter_mut()
                    .find(|x| other_shader_resource.name == x.name);
                match found {
                    Some(shader_resource) => {
                        if shader_resource.shader_resource_type
                            == other_shader_resource.shader_resource_type
                            && shader_resource.binding == other_shader_resource.binding
                            && shader_resource.element_count == other_shader_resource.element_count
                        {
                            shader_resource.used_in_shader_stages |=
                                other_shader_resource.used_in_shader_stages;
                        } else {
                            let message =
                                "Cannot merge shader resource of different size".to_owned();
                            log::error!("{}", message);
                            return Err(message.into());
                        }
                    }
                    None => {
                        result.push(other_shader_resource.clone());
                    }
                }
            }
        } else {
            result = reflection.shader_resources.clone();
        }
    }

    Ok(result)
}
