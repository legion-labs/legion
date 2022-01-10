use std::collections::HashSet;

use lgn_graphics_api::{ShaderResourceType, MAX_DESCRIPTOR_SET_LAYOUTS};
use strum::EnumString;

use super::{CGenTypeHandle, ModelHandle, ModelObject};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextureDef {
    pub ty_ref: CGenTypeHandle,
}

#[derive(Debug)]
pub struct SamplerDescriptorDef;

#[derive(Debug, EnumString)]
pub enum DescriptorType {
    Sampler,
    ConstantBuffer,
    StructuredBuffer,
    RWStructuredBuffer,
    ByteAddressBuffer,
    RWByteAddressBuffer,
    Texture2D,
    RWTexture2D,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConstantBufferDef {
    pub ty_handle: CGenTypeHandle,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StructuredBufferDef {
    pub ty_handle: CGenTypeHandle,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DescriptorDef {
    // Sampler
    Sampler,
    // Buffers
    ConstantBuffer(ConstantBufferDef),
    StructuredBuffer(StructuredBufferDef),
    RWStructuredBuffer(StructuredBufferDef),
    ByteAddressBuffer,
    RWByteAddressBuffer,
    // Textures
    Texture2D(TextureDef),
    RWTexture2D(TextureDef),
    Texture3D(TextureDef),
    RWTexture3D(TextureDef),
    Texture2DArray(TextureDef),
    RWTexture2DArray(TextureDef),
    TextureCube(TextureDef),
    TextureCubeArray(TextureDef),
}

impl DescriptorDef {
    pub fn shader_resource_type(&self) -> ShaderResourceType {
        match self {
            crate::db::DescriptorDef::Sampler => ShaderResourceType::Sampler,
            crate::db::DescriptorDef::ConstantBuffer(_) => ShaderResourceType::ConstantBuffer,
            crate::db::DescriptorDef::StructuredBuffer(_) => ShaderResourceType::StructuredBuffer,
            crate::db::DescriptorDef::RWStructuredBuffer(_) => {
                ShaderResourceType::RWStructuredBuffer
            }
            crate::db::DescriptorDef::ByteAddressBuffer => ShaderResourceType::ByteAdressBuffer,
            crate::db::DescriptorDef::RWByteAddressBuffer => ShaderResourceType::RWByteAdressBuffer,
            crate::db::DescriptorDef::Texture2D(_) => ShaderResourceType::Texture2D,
            crate::db::DescriptorDef::RWTexture2D(_) => ShaderResourceType::RWTexture2D,
            crate::db::DescriptorDef::Texture3D(_) => ShaderResourceType::Texture3D,
            crate::db::DescriptorDef::RWTexture3D(_) => ShaderResourceType::RWTexture3D,
            crate::db::DescriptorDef::Texture2DArray(_) => ShaderResourceType::Texture2DArray,
            crate::db::DescriptorDef::RWTexture2DArray(_) => ShaderResourceType::RWTexture2DArray,
            crate::db::DescriptorDef::TextureCube(_) => ShaderResourceType::TextureCube,
            crate::db::DescriptorDef::TextureCubeArray(_) => ShaderResourceType::TextureCubeArray,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Descriptor {
    pub name: String,
    pub flat_index: u32,
    pub array_len: Option<u32>,
    pub def: DescriptorDef,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DescriptorSet {
    pub name: String,
    pub frequency: u32,
    pub flat_descriptor_count: u32,
    pub descriptors: Vec<Descriptor>,
}

pub type DescriptorSetHandle = ModelHandle<DescriptorSet>;

impl DescriptorSet {
    pub fn new(name: &str, frequency: u32) -> Self {
        assert!((frequency as usize) < MAX_DESCRIPTOR_SET_LAYOUTS);
        Self {
            name: name.to_owned(),
            frequency,
            descriptors: Vec::new(),
            flat_descriptor_count: 0,
        }
    }

    pub fn get_type_dependencies(&self) -> HashSet<CGenTypeHandle> {
        let mut set = HashSet::new();

        for descriptor in &self.descriptors {
            match &descriptor.def {
                crate::db::DescriptorDef::ConstantBuffer(def) => {
                    set.insert(def.ty_handle);
                }
                crate::db::DescriptorDef::StructuredBuffer(def)
                | crate::db::DescriptorDef::RWStructuredBuffer(def) => {
                    set.insert(def.ty_handle);
                }
                crate::db::DescriptorDef::Sampler
                | crate::db::DescriptorDef::ByteAddressBuffer
                | crate::db::DescriptorDef::RWByteAddressBuffer
                | crate::db::DescriptorDef::Texture2D(_)
                | crate::db::DescriptorDef::RWTexture2D(_)
                | crate::db::DescriptorDef::Texture3D(_)
                | crate::db::DescriptorDef::RWTexture3D(_)
                | crate::db::DescriptorDef::Texture2DArray(_)
                | crate::db::DescriptorDef::RWTexture2DArray(_)
                | crate::db::DescriptorDef::TextureCube(_)
                | crate::db::DescriptorDef::TextureCubeArray(_) => (),
            }
        }
        set
    }
}

impl ModelObject for DescriptorSet {
    fn typename() -> &'static str {
        "DescriptorSet"
    }
    fn name(&self) -> &str {
        self.name.as_str()
    }
}
