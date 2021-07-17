use crate::types::{ResourceType, ShaderStageFlags};
use crate::{Api, GfxResult, ShaderStageDef, MAX_DESCRIPTOR_SET_LAYOUTS};
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
/// A `ShaderResource` may be specified by hand or generated using rafx-shader-processor
//TODO: Consider separate type for bindings vs. push constants
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub struct ShaderResource {
    pub resource_type: ResourceType,
    pub set_index: u32,
    pub binding: u32,
    // Valid only for descriptors (resource_type != ROOT_CONSTANT)
    // This must remain pub to init the struct as "normal" but in general,
    // access it via element_count_normalized(). This ensures that if it
    // is default-initialized to 0, it is treated as 1
    pub element_count: u32,
    // Valid only for push constants (resource_type != ROOT_CONSTANT)
    pub size_in_bytes: u32,
    pub used_in_shader_stages: ShaderStageFlags,
    // Name is optional
    // TODO: Add some sort of hashing-friendly option
    pub name: Option<String>,
}

impl ShaderResource {
    pub fn element_count_normalized(&self) -> u32 {
        // Assume 0 = default of 1
        self.element_count.max(1)
    }

    pub fn validate(&self) -> GfxResult<()> {
        if self.resource_type == ResourceType::ROOT_CONSTANT {
            if self.element_count != 0 {
                return Err(format!(
                        "binding (set={:?} binding={:?} name={:?} type={:?}) has non-zero element_count",
                        self.set_index,
                        self.binding,
                        self.name,
                        self.resource_type
                    ).into());
            }
            if self.size_in_bytes == 0 {
                return Err(format!(
                    "binding (set={:?} binding={:?} name={:?} type={:?}) has zero size_in_bytes",
                    self.set_index, self.binding, self.name, self.resource_type
                )
                .into());
            }
            if self.set_index != 0 {
                return Err(format!(
                    "binding (set={:?} binding={:?} name={:?} type={:?}) has non-zero set_index",
                    self.set_index, self.binding, self.name, self.resource_type
                )
                .into());
            }
            if self.binding != 0 {
                return Err(format!(
                    "binding (set={:?} binding={:?} name={:?} type={:?}) has non-zero binding",
                    self.set_index, self.binding, self.name, self.resource_type
                )
                .into());
            }
        } else {
            if self.size_in_bytes != 0 {
                return Err(format!(
                        "binding (set={:?} binding={:?} name={:?} type={:?}) has non-zero size_in_bytes",
                        self.set_index,
                        self.binding,
                        self.name,
                        self.resource_type
                    ).into());
            }

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
        if self.resource_type != other.resource_type {
            return Err(format!(
                "Pass is using shaders in different stages with different resource_type {:?} and {:?} (set={} binding={})",
                self.resource_type, other.resource_type,
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

        if self.size_in_bytes != other.size_in_bytes {
            return Err(format!(
                "Pass is using shaders in different stages with different size_in_bytes {} and {} (set={} binding={})",
                self.size_in_bytes, other.size_in_bytes,
                self.set_index, self.binding
            ).into());
        }

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
    pub resources: Vec<ShaderResource>,
    pub compute_threads_per_group: Option<[u32; 3]>,
    pub entry_point_name: String,
    // Right now we will infer mappings based on spirv_cross default behavior, but likely will want
    // to allow providing them explicitly. This isn't implemented yet
    //pub binding_arg_buffer_mappings: FnvHashMap<(u32, u32), u32>
}

/// Reflection data for a pipeline, created by merging shader stage reflection data
#[derive(Debug)]
pub struct PipelineReflection {
    pub shader_stages: ShaderStageFlags,
    pub resources: Vec<ShaderResource>,
    pub compute_threads_per_group: Option<[u32; 3]>,
}

impl PipelineReflection {
    pub fn from_stages<A: Api>(stages: &[ShaderStageDef<A>]) -> GfxResult<Self> {
        let mut unmerged_resources = Vec::default();
        for stage in stages {
            assert!(!stage.reflection.shader_stage.is_empty());
            for resource in &stage.reflection.resources {
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

        //TODO: Merge push constants

        //
        // Merge the resources
        //
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

        let resources = merged_resources.into_iter().map(|(_, v)| v).collect();

        Ok(Self {
            shader_stages: all_shader_stages,
            compute_threads_per_group,
            resources,
        })
    }
}
