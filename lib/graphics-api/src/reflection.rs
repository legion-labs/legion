use crate::types::{ShaderStageFlags};
use crate::{GfxApi, GfxResult, MAX_DESCRIPTOR_SET_LAYOUTS, ShaderResourceType, ShaderStageDef};
use fnv::FnvHashMap;
#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};

// #[derive(Debug, PartialEq)]
// pub enum ShaderResourceType {
//     Sampler,
//     ConstBuffer,
//     StructuredBuffer,
//     RawByteBuffer
// }

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
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub struct ShaderResource {
    pub name: String,
    pub shader_resource_type: ShaderResourceType,
    pub binding: u32,    
    pub set_index: u32,
    pub element_count: u32,    
    pub used_in_shader_stages: ShaderStageFlags,    
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct PushConstant {
    pub used_in_shader_stages: ShaderStageFlags,
    pub size: u32
}

impl PushConstant {
    pub(crate) fn verify_compatible_across_stages(&self, other: &Self) -> GfxResult<()> {
        if self.size != other.size {
            return Err(format!(
                "PushConstant has different size in different stages"                
            ).into());
        }       

        Ok(())
    }
}

impl ShaderResource {
    pub fn element_count_normalized(&self) -> u32 {
        // Assume 0 = default of 1
        self.element_count.max(1)
    }

    pub fn validate(&self) -> GfxResult<()> {
        // if self.resource_type == ResourceType::ROOT_CONSTANT {
        //     if self.element_count != 0 {
        //         return Err(format!(
        //                 "binding (set={:?} binding={:?} name={:?} type={:?}) has non-zero element_count",
        //                 self.set_index,
        //                 self.binding,
        //                 self.name,
        //                 self.resource_type
        //             ).into());
        //     }
        //     if self.size_in_bytes == 0 {
        //         return Err(format!(
        //             "binding (set={:?} binding={:?} name={:?} type={:?}) has zero size_in_bytes",
        //             self.set_index, self.binding, self.name, self.resource_type
        //         )
        //         .into());
        //     }
        //     if self.set_index != 0 {
        //         return Err(format!(
        //             "binding (set={:?} binding={:?} name={:?} type={:?}) has non-zero set_index",
        //             self.set_index, self.binding, self.name, self.resource_type
        //         )
        //         .into());
        //     }
        //     if self.binding != 0 {
        //         return Err(format!(
        //             "binding (set={:?} binding={:?} name={:?} type={:?}) has non-zero binding",
        //             self.set_index, self.binding, self.name, self.resource_type
        //         )
        //         .into());
        //     }
        // } else 
        {
            // if self.size_in_bytes != 0 {
            //     return Err(format!(
            //             "binding (set={:?} binding={:?} name={:?} type={:?}) has non-zero size_in_bytes",
            //             self.set_index,
            //             self.binding,
            //             self.name,
            //             self.shader_resource_type
            //         ).into());
            // }

            if self.set_index as usize >= MAX_DESCRIPTOR_SET_LAYOUTS {
                return Err(format!(
                    "Descriptor (set={:?} binding={:?}) named {:?} has a set index >= 4. This is not supported",
                    self.set_index, self.binding, self.name,
                ).into());
            }
        }

        Ok(())
    }

    fn binding_key(&self) -> ShaderResourceBindingKey {
        ShaderResourceBindingKey {
            set: self.set_index,
            binding: self.binding,
        }
    }

    fn verify_compatible_across_stages(&self, other: &Self) -> GfxResult<()> {
        if self.shader_resource_type != other.shader_resource_type {
            return Err(format!(
                "Pass is using shaders in different stages with different resource_type {:?} and {:?} (set={} binding={})",
                self.shader_resource_type, other.shader_resource_type,
                self.set_index,
                self.binding
            ).into());
        }

        if self.element_count_normalized() != other.element_count_normalized() {
            return Err(format!(
                "Pass is using shaders in different stages with different element_count {} and {} (set={} binding={})", self.element_count_normalized(), other.element_count_normalized(),
                self.set_index, self.binding
            ).into());
        }

        // if self.size_in_bytes != other.size_in_bytes {
        //     return Err(format!(
        //         "Pass is using shaders in different stages with different size_in_bytes {} and {} (set={} binding={})",
        //         self.size_in_bytes, other.size_in_bytes,
        //         self.set_index, self.binding
        //     ).into());
        // }

        Ok(())
    }
}

/// Reflection data for a single shader stage
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub struct ShaderStageReflection {
    // For now, this doesn't do anything, so commented out
    //pub vertex_inputs: Vec<VertexInput>,
    pub shader_stage: ShaderStageFlags,
    pub shader_resources: Vec<ShaderResource>,
    pub push_constants: Vec<PushConstant>,
    pub compute_threads_per_group: Option<[u32; 3]>,
    pub entry_point_name: String,
}

/// Reflection data for a pipeline, created by merging shader stage reflection data
#[derive(Debug)]
pub struct PipelineReflection {
    pub shader_stages: ShaderStageFlags,
    pub shader_resources: Vec<ShaderResource>,
    pub push_constant: Option<PushConstant>,
    pub compute_threads_per_group: Option<[u32; 3]>,
}

impl PipelineReflection {
    pub fn from_stages<A: GfxApi>(stages: &[ShaderStageDef<A>]) -> GfxResult<Self> {        
        
        let mut unmerged_resources = Vec::default();
        for stage in stages {
            assert!(!stage.reflection.shader_stage.is_empty());
            for resource in &stage.reflection.shader_resources {
                // The provided resource MAY (but does not need to) have the shader stage flag set.
                // (Leaving it default empty is fine). It will automatically be set here.
                if !(resource.used_in_shader_stages - stage.reflection.shader_stage).is_empty() {
                    let message = format!(
                        "A resource in shader stage {:?} has other stages {:?} set",
                        stage.reflection.shader_stage,
                        resource.used_in_shader_stages - stage.reflection.shader_stage
                    );
                    log::error!("{}", message);
                    return Err(message.into());
                }

                let mut resource = resource.clone();
                resource.used_in_shader_stages |= stage.reflection.shader_stage;
                unmerged_resources.push(resource);
            }
        }

        let mut unmerged_pushconstants = Vec::default();
        for stage in stages {
            assert!(!stage.reflection.shader_stage.is_empty());
            for push_constant in &stage.reflection.push_constants {
                // The provided resource MAY (but does not need to) have the shader stage flag set.
                // (Leaving it default empty is fine). It will automatically be set here.
                if !(push_constant.used_in_shader_stages - stage.reflection.shader_stage).is_empty() {
                    let message = format!(
                        "A resource in shader stage {:?} has other stages {:?} set",
                        stage.reflection.shader_stage,
                        push_constant.used_in_shader_stages - stage.reflection.shader_stage
                    );
                    log::error!("{}", message);
                    return Err(message.into());
                }

                let mut push_constant = push_constant.clone();
                push_constant.used_in_shader_stages |= stage.reflection.shader_stage;
                unmerged_pushconstants.push(push_constant);
            }
        }

        let mut compute_threads_per_group = None;
        for stage in stages {
            if stage
                .reflection
                .shader_stage
                .intersects(ShaderStageFlags::COMPUTE)
            {
                compute_threads_per_group = stage.reflection.compute_threads_per_group;
            }
        }

        log::trace!("Create PipelineReflection from stages");
        let mut all_shader_stages = ShaderStageFlags::empty();
        for stage in stages {
            if all_shader_stages.intersects(stage.reflection.shader_stage) {
                return Err(format!(
                    "Duplicate shader stage ({}) found when creating PipelineReflection",
                    (all_shader_stages & stage.reflection.shader_stage).bits()
                )
                .into());
            }

            all_shader_stages |= stage.reflection.shader_stage;
        }

        let mut merged_resources =
            FnvHashMap::<ShaderResourceBindingKey, ShaderResource>::default();               
        for resource in &unmerged_resources {
            log::trace!(
                "    Resource {:?} from stage {:?}",
                resource.name,
                resource.used_in_shader_stages
            );
            let key = resource.binding_key();
            if let Some(existing_resource) = merged_resources.get_mut(&key) {
                // verify compatible
                existing_resource.verify_compatible_across_stages(resource)?;

                log::trace!(
                    "      Already used in stages {:?} and is compatible, adding stage {:?}",
                    existing_resource.used_in_shader_stages,
                    resource.used_in_shader_stages,
                );
                existing_resource.used_in_shader_stages |= resource.used_in_shader_stages;
            } else {
                // insert it
                log::trace!(
                    "      Resource not yet used, adding it for stage {:?}",
                    resource.used_in_shader_stages
                );
                assert!(!resource.used_in_shader_stages.is_empty());
                let old = merged_resources.insert(key, resource.clone());
                assert!(old.is_none());
            }
        }

        let mut merged_pushconstant : Option<PushConstant> = None;
        for push_constant in unmerged_pushconstants {                        
            log::trace!(
                "    PushConstant from stage {:?}",                
                push_constant.used_in_shader_stages
            );            
            if let Some(existing_push_constant) = &mut merged_pushconstant {
                // verify compatible
                existing_push_constant.verify_compatible_across_stages(&push_constant)?;

                log::trace!(
                    "      Already used in stages {:?} and is compatible, adding stage {:?}",
                    existing_push_constant.used_in_shader_stages,
                    push_constant.used_in_shader_stages,
                );
                existing_push_constant.used_in_shader_stages |= push_constant.used_in_shader_stages;
            } else {
                // insert it
                log::trace!(
                    "      Resource not yet used, adding it for stage {:?}",
                    push_constant.used_in_shader_stages
                );
                assert!(!push_constant.used_in_shader_stages.is_empty());
                merged_pushconstant = Some(push_constant);                
            }
        }

        let shader_resources = merged_resources.into_iter().map(|(_, v)| v).collect();
        let push_constant = merged_pushconstant;

        Ok(Self {
            shader_stages: all_shader_stages,
            compute_threads_per_group,
            shader_resources,
            push_constant
        })
    }
}
