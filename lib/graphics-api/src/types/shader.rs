use std::{hash::Hash, marker::PhantomData};

#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};

/// Owns data necessary to create a shader module in (optionally) multiple APIs.
///
/// This struct can be serialized/deserialized and is intended to allow asset pipeline to store
/// a shader module to be created at runtime. The package can optionally include data for multiple
/// APIs allowing a single file to be used with whatever API is found at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum ShaderPackage {
    SpirV(Vec<u8>),
    Null,
}

impl ShaderPackage {
    pub fn module_def(&self) -> ShaderModuleDef<'_> {
        match self {
            ShaderPackage::SpirV(bytes) => ShaderModuleDef::SpirVBytes(bytes),
            ShaderPackage::Null => ShaderModuleDef::Null(PhantomData::default()),
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
