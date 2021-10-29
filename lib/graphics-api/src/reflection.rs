use crate::types::ShaderStageFlags;
use crate::{GfxResult, ShaderResourceType, MAX_DESCRIPTOR_SET_LAYOUTS};

use fnv::FnvHashMap;
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

impl PushConstant {
    pub(crate) fn verify_compatible_across_stages(self, other: Self) -> GfxResult<()> {
        if self.size != other.size {
            return Err("PushConstant has different size in different stages".into());
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
        if self.set_index as usize >= MAX_DESCRIPTOR_SET_LAYOUTS {
            return Err(format!(
                "Descriptor (set={:?} binding={:?}) named {:?} has a set index >= 4. This is not supported",
                self.set_index, self.binding, self.name,
            ).into());
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

        Ok(())
    }
}

/// Reflection data for a single shader stage
// #[derive(Debug, Clone, PartialEq)]
// #[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
// pub struct ShaderStageReflection {
//     // pub shader_stage: ShaderStageFlags,
//     pub shader_resources: Vec<ShaderResource>,
//     pub push_constants: Vec<PushConstant>,
//     pub compute_threads_per_group: Option<[u32; 3]>,
//     // pub entry_point_name: String,
// }

// impl Default for ShaderStageReflection {
//     fn default() -> Self {
//         Self {
//             // shader_stage: ShaderStageFlags::empty(),
//             shader_resources: Vec::new(),
//             push_constants: Vec::new(),
//             compute_threads_per_group: None,
//             // entry_point_name: String::new(),
//         }
//     }
// }

/// Reflection data for a pipeline, created by merging shader stage reflection data
#[derive(Clone, Debug)]
pub struct PipelineReflection {
    // pub shader_stages: ShaderStageFlags,
    pub shader_resources: Vec<ShaderResource>,
    pub push_constant: Option<PushConstant>,
    pub compute_threads_per_group: Option<[u32; 3]>,
}

impl Default for PipelineReflection {
    fn default() -> Self {
        PipelineReflection {
            // shader_stages: ShaderStageFlags::empty(),
            shader_resources: Vec::new(),
            push_constant: None,
            compute_threads_per_group: None,
        }
    }
}

impl PipelineReflection {
    // pub fn from_stages<A: GfxApi>(stages: &[ShaderStageDef<A>]) -> GfxResult<Self> {
    //     let shader_stages = all_shader_stages(stages)?;
    //     let shader_resources = merge_resources(stages)?;
    //     let push_constant = merge_pushconstant(stages)?;
    //     let compute_threads_per_group = compute_threads_per_group(stages);

    //     Ok(Self {
    //         shader_stages,
    //         shader_resources,
    //         push_constant,
    //         compute_threads_per_group,
    //     })
    // }
    pub fn merge(
        left_op: &PipelineReflection,
        right_op: &PipelineReflection,
    ) -> GfxResult<PipelineReflection> {
        let arr = [left_op, right_op];        
        Ok(
            PipelineReflection{
                shader_resources: merge_resources(&arr).unwrap(),
                push_constant: merge_pushconstant(&arr).unwrap(),
                compute_threads_per_group: None,
            }
        )
    }
}

fn merge_pushconstant(reflections: &[&PipelineReflection]) -> GfxResult<Option<PushConstant>> {

    let mut result: Option<PushConstant> = None;

    for reflection in reflections {
        if let Some(push_constant) = &mut result {
            if let Some(other_push_constant) = reflection.push_constant {
                if push_constant.size != other_push_constant.size {
                    let message = format!(
                        "Cannot merge pushconstants of different size",
                    );
                    log::error!("{}", message);
                    return Err(message.into()); 
                }
                // let mut merged_push_constant = push_constant;
                // merged_push_constant.used_in_shader_stages |= other_push_constant.used_in_shader_stages;
                // result = Some(merged_push_constant);
                push_constant.used_in_shader_stages |= other_push_constant.used_in_shader_stages;

            }
        } else {
            result = reflection.push_constant
        }        
    }

    Ok(result)
/*
    let mut unmerged_pushconstants = Vec::default();
    for reflection in reflections {
        assert!(!reflection.shader_stage.is_empty());
        for push_constant in &reflection.push_constants {
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

            let mut push_constant = *push_constant;
            push_constant.used_in_shader_stages |= stage.reflection.shader_stage;
            unmerged_pushconstants.push(push_constant);
        }
    }
    let mut merged_pushconstant: Option<PushConstant> = None;
    for push_constant in unmerged_pushconstants {
        log::trace!(
            "    PushConstant from stage {:?}",
            push_constant.used_in_shader_stages
        );
        if let Some(existing_push_constant) = &mut merged_pushconstant {
            // verify compatible
            existing_push_constant.verify_compatible_across_stages(push_constant)?;

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
    Ok(merged_pushconstant)
    */
}

fn merge_resources(reflections: &[&PipelineReflection]) -> GfxResult<Vec<ShaderResource>> {

    let mut result = Vec::<ShaderResource>::new();

    for reflection in reflections {
        if !result.is_empty() {
            for other_shader_resource in &reflection.shader_resources {
                let found = result.iter_mut().find(
                    |x| other_shader_resource.name == x.name
                );
                if found.is_none() {
                    result.push(other_shader_resource.clone());
                } else {
                    let shader_resource = found.unwrap();
                    if  shader_resource.shader_resource_type == other_shader_resource.shader_resource_type && 
                        shader_resource.binding == other_shader_resource.binding && 
                        shader_resource.element_count == other_shader_resource.element_count 
                    {      
                        shader_resource.used_in_shader_stages |= other_shader_resource.used_in_shader_stages;
                    } else {
                        let message = format!(
                            "Cannot merge shader resource of different size",
                        );
                        log::error!("{}", message);
                        return Err(message.into()); 
                    }
                }
            }            
        } else {
            result = reflection.shader_resources.clone();
        }        
    }

    Ok(result)

    // let mut unmerged_resources = Vec::default();
    // for stage in stages {
    //     assert!(!stage.reflection.shader_stage.is_empty());
    //     for resource in &stage.reflection.shader_resources {
    //         // The provided resource MAY (but does not need to) have the shader stage flag set.
    //         // (Leaving it default empty is fine). It will automatically be set here.
    //         if !(resource.used_in_shader_stages - stage.reflection.shader_stage).is_empty() {
    //             let message = format!(
    //                 "A resource in shader stage {:?} has other stages {:?} set",
    //                 stage.reflection.shader_stage,
    //                 resource.used_in_shader_stages - stage.reflection.shader_stage
    //             );
    //             log::error!("{}", message);
    //             return Err(message.into());
    //         }

    //         let mut resource = resource.clone();
    //         resource.used_in_shader_stages |= stage.reflection.shader_stage;
    //         unmerged_resources.push(resource);
    //     }
    // }
    // let mut merged_resources = FnvHashMap::<ShaderResourceBindingKey, ShaderResource>::default();
    // for resource in &unmerged_resources {
    //     log::trace!(
    //         "    Resource {:?} from stage {:?}",
    //         resource.name,
    //         resource.used_in_shader_stages
    //     );
    //     let key = resource.binding_key();
    //     if let Some(existing_resource) = merged_resources.get_mut(&key) {
    //         // verify compatible
    //         existing_resource.verify_compatible_across_stages(resource)?;

    //         log::trace!(
    //             "      Already used in stages {:?} and is compatible, adding stage {:?}",
    //             existing_resource.used_in_shader_stages,
    //             resource.used_in_shader_stages,
    //         );
    //         existing_resource.used_in_shader_stages |= resource.used_in_shader_stages;
    //     } else {
    //         // insert it
    //         log::trace!(
    //             "      Resource not yet used, adding it for stage {:?}",
    //             resource.used_in_shader_stages
    //         );
    //         assert!(!resource.used_in_shader_stages.is_empty());
    //         let old = merged_resources.insert(key, resource.clone());
    //         assert!(old.is_none());
    //     }
    // }

    // Ok(merged_resources.into_iter().map(|(_, v)| v).collect())
}

// fn all_shader_stages<A: GfxApi>(stages: &[ShaderStageDef<A>]) -> GfxResult<ShaderStageFlags> {
//     let mut all_shader_stages = ShaderStageFlags::empty();
//     for stage in stages {
//         if all_shader_stages.intersects(stage.reflection.shader_stage) {
//             return Err(format!(
//                 "Duplicate shader stage ({}) found when creating PipelineReflection",
//                 (all_shader_stages & stage.reflection.shader_stage).bits()
//             )
//             .into());
//         }

//         all_shader_stages |= stage.reflection.shader_stage;
//     }
//     Ok(all_shader_stages)
// }

// fn compute_threads_per_group<A: GfxApi>(stages: &[ShaderStageDef<A>]) -> Option<[u32; 3]> {
//     let mut compute_threads_per_group = None;
//     for stage in stages {
//         if stage
//             .reflection
//             .shader_stage
//             .intersects(ShaderStageFlags::COMPUTE)
//         {
//             compute_threads_per_group = stage.reflection.compute_threads_per_group;
//         }
//     }
//     compute_threads_per_group
// }
