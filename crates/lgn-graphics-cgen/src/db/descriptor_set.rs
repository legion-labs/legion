use std::collections::HashSet;

use lgn_graphics_api::{ShaderResourceType, MAX_DESCRIPTOR_SET_LAYOUTS};
use strum::EnumString;

use super::{CGenType, CGenTypeHandle, Model, ModelHandle, ModelObject, NativeType};

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextureDef {
    pub ty_handle: CGenTypeHandle,
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
            crate::db::DescriptorDef::ByteAddressBuffer => ShaderResourceType::ByteAddressBuffer,
            crate::db::DescriptorDef::RWByteAddressBuffer => {
                ShaderResourceType::RWByteAddressBuffer
            }
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
    pub bindless: bool,
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

pub struct DescriptorSetBuilder<'mdl> {
    mdl: &'mdl Model,
    product: DescriptorSet,
    names: HashSet<String>,
    flat_index: u32,
}

impl<'mdl> DescriptorSetBuilder<'mdl> {
    pub fn new(mdl: &'mdl Model, name: &str, frequency: u32) -> Self {
        DescriptorSetBuilder {
            mdl,
            product: DescriptorSet::new(name, frequency),
            names: HashSet::new(),
            flat_index: 0,
        }
    }

    /// Add Samplers.
    ///
    /// # Errors
    /// todo
    pub fn add_samplers(self, name: &str, array_len: Option<u32>) -> Result<Self> {
        self.add_descriptor(name, false, array_len, DescriptorDef::Sampler)
    }

    /// Add `ConstantBuffers`.
    ///
    /// # Errors
    /// todo
    pub fn add_constant_buffer(self, name: &str, inner_type: &str) -> Result<Self> {
        // get cgen type and check its existence if necessary
        let ty_handle = self
            .mdl
            .get_object_handle::<CGenType>(inner_type)
            .with_context(|| {
                anyhow!(
                    "ConstantBuffer '{}' has an unknown type '{}'",
                    name,
                    inner_type
                )
            })?;
        self.add_descriptor(
            name,
            false,
            None,
            DescriptorDef::ConstantBuffer(ConstantBufferDef { ty_handle }),
        )
    }

    /// Add `StructuredBuffers`.
    ///
    /// # Errors
    /// todo
    pub fn add_structured_buffer(
        self,
        name: &str,
        array_len: Option<u32>,
        inner_ty: &str,
        read_write: bool,
    ) -> Result<Self> {
        // get cgen type and check its existence if necessary
        let ty_handle = self
            .mdl
            .get_object_handle::<CGenType>(inner_ty)
            .with_context(|| {
                anyhow!(
                    "StructuredBuffer '{}' has an unknown type '{}'",
                    name,
                    inner_ty
                )
            })?;
        let def = StructuredBufferDef { ty_handle };
        let def = if read_write {
            DescriptorDef::RWStructuredBuffer(def)
        } else {
            DescriptorDef::StructuredBuffer(def)
        };
        self.add_descriptor(name, false, array_len, def)
    }

    /// Add `ByteAddressBuffer`.
    ///
    /// # Errors
    /// todo
    pub fn add_byte_address_buffer(
        self,
        name: &str,
        array_len: Option<u32>,
        read_write: bool,
    ) -> Result<Self> {
        let def = if read_write {
            DescriptorDef::RWByteAddressBuffer
        } else {
            DescriptorDef::ByteAddressBuffer
        };
        self.add_descriptor(name, false, array_len, def)
    }

    /// Add descriptor.
    ///
    /// # Errors
    /// todo
    pub fn add_texture(
        self,
        name: &str,
        tex_type: &str,
        fmt: &str,
        bindless: bool,
        array_len: Option<u32>,
        read_write: bool,
    ) -> Result<Self> {
        //
        // Texture format
        //
        let ty_handle = self
            .mdl
            .get_object_handle::<CGenType>(fmt)
            .with_context(|| anyhow!("Texture '{}' has an unknown type '{}'", name, fmt))?;
        let fmt_ty = ty_handle.get(self.mdl);
        let valid_type = match fmt_ty {
            CGenType::Struct(_) | CGenType::BitField(_) => false,
            CGenType::Native(e) => matches!(e, NativeType::Float(_)),
        };
        if !valid_type {
            return Err(anyhow!(
                "Format type '{}' for Texture '{}' is not valid",
                fmt,
                name
            ));
        }
        let def = TextureDef { ty_handle };
        let ds = match tex_type {
            "2D" => {
                if read_write {
                    DescriptorDef::RWTexture2D(def)
                } else {
                    DescriptorDef::Texture2D(def)
                }
            }
            "3D" => {
                if read_write {
                    DescriptorDef::RWTexture3D(def)
                } else {
                    DescriptorDef::Texture3D(def)
                }
            }
            "2DArray" => {
                if read_write {
                    DescriptorDef::RWTexture2DArray(def)
                } else {
                    DescriptorDef::Texture2DArray(def)
                }
            }
            "Cube" => {
                if read_write {
                    return Err(anyhow!(
                        "Texture type '{}' for Texture '{}' cant be writable",
                        tex_type,
                        name,
                    ));
                }
                DescriptorDef::TextureCube(def)
            }
            "CubeArray" => {
                if read_write {
                    return Err(anyhow!(
                        "Texture type '{}'for Texture '{}' cant be writable",
                        tex_type,
                        name,
                    ));
                }
                DescriptorDef::TextureCubeArray(def)
            }
            _ => {
                return Err(anyhow!(
                    "Texture type '{}'for Texture '{}' is not valid",
                    tex_type,
                    name,
                ));
            }
        };

        self.add_descriptor(name, bindless, array_len, ds)
    }

    fn add_descriptor(
        mut self,
        name: &str,
        bindless: bool,
        array_len: Option<u32>,
        def: DescriptorDef,
    ) -> Result<Self> {
        if let Some(array_len) = array_len {
            if array_len == 0 {
                return Err(anyhow!("Descriptor '{}' have array len set to 0", name,));
            }
        }

        if self.names.contains(name) {
            return Err(anyhow!("Descriptor '{}' already exists", name,));
        }
        self.names.insert(name.to_string());
        self.product.descriptors.push(Descriptor {
            name: name.to_owned(),
            bindless,
            flat_index: self.flat_index,
            array_len,
            def,
        });

        self.flat_index += array_len.unwrap_or(1u32);

        Ok(self)
    }

    /// Build.
    ///
    /// # Errors
    /// todo
    #[allow(clippy::unnecessary_wraps)]
    pub fn build(mut self) -> Result<DescriptorSet> {
        self.product.flat_descriptor_count = self.flat_index;
        Ok(self.product)
    }
}
