use std::{hash::Hash, marker::PhantomData};

use crate::{deferred_drop::Drc, DeviceContext, ShaderModule, ShaderStage, ShaderStageFlags};

/// Describes a single stage within a shader
#[derive(Clone, PartialEq)]
pub struct ShaderStageDef {
    pub entry_point: String,
    pub shader_stage: ShaderStage,
    pub shader_module: ShaderModule,
}

/// Owns data necessary to create a shader module in (optionally) multiple APIs.
///
/// This struct can be serialized/deserialized and is intended to allow asset
/// pipeline to store a shader module to be created at runtime. The package can
/// optionally include data for multiple APIs allowing a single file to be used
/// with whatever API is found at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShaderPackage {
    SpirV(Vec<u8>),
    Null,
}

impl ShaderPackage {
    pub fn module_def(&self) -> ShaderModuleDef<'_> {
        match self {
            Self::SpirV(bytes) => ShaderModuleDef::SpirVBytes(bytes),
            Self::Null => ShaderModuleDef::Null(PhantomData::default()),
        }
    }
}

/// Used to create a `ShaderModule`
///
/// This enum may be populated manually or created from a `ShaderPackage`.
#[derive(Copy, Clone, Hash)]
pub enum ShaderModuleDef<'a> {
    /// Raw SPV bytes, no alignment or endianness requirements.
    SpirVBytes(&'a [u8]),
    Null(std::marker::PhantomData<&'a u8>),
}

pub(crate) struct ShaderInner {
    stage_flags: ShaderStageFlags,
    stages: Vec<ShaderStageDef>,
}

#[derive(Clone)]
pub struct Shader {
    inner: Drc<ShaderInner>,
}

impl PartialEq for Shader {
    fn eq(&self, other: &Self) -> bool {
        self.inner.stage_flags == other.inner.stage_flags && self.inner.stages == other.inner.stages
    }
}

impl Shader {
    pub fn new(device_context: &DeviceContext, stages: Vec<ShaderStageDef>) -> Self {
        let mut stage_flags = ShaderStageFlags::empty();
        for stage in &stages {
            stage_flags |= stage.shader_stage.into();
        }

        let inner = ShaderInner {
            stage_flags,
            stages,
        };

        Self {
            inner: device_context.deferred_dropper().new_drc(inner),
        }
    }

    pub fn stages(&self) -> &[ShaderStageDef] {
        &self.inner.stages
    }

    pub fn stage_flags(&self) -> ShaderStageFlags {
        self.inner.stage_flags
    }
}
