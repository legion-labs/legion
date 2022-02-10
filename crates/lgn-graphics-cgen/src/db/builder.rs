use std::collections::HashSet;

use anyhow::{anyhow, Context, Result};

use crate::db::{
    CGenType, ConstantBufferDef, Descriptor, DescriptorDef, DescriptorSet, Model, NativeType,
    PipelineLayout, StructMember, StructType, StructuredBufferDef, TextureDef,
};

pub struct StructBuilder<'mdl> {
    mdl: &'mdl Model,
    product: StructType,
    names: HashSet<String>,
}

impl<'mdl> StructBuilder<'mdl> {
    pub fn new(mdl: &'mdl Model, name: &str) -> Self {
        StructBuilder {
            mdl,
            product: StructType::new(name),
            names: HashSet::new(),
        }
    }

    /// Add struct member
    ///
    /// # Errors
    /// todo
    pub fn add_member(mut self, name: &str, typ: &str, array_len: Option<u32>) -> Result<Self> {
        // check member uniqueness
        if self.names.contains(name) {
            return Err(anyhow!(
                "Member '{}' already exists in struct '{}'",
                name,
                self.product.name
            ));
        }
        self.names.insert(name.to_string());

        // check array_len validity
        if let Some(array_len) = array_len {
            if array_len == 0 {
                return Err(anyhow!(
                    "Member '{}' in struct '{}' can't have a zero array_len",
                    name,
                    self.product.name
                ));
            }
        }

        // get cgen type and check its existence if necessary
        let ty_ref = self
            .mdl
            .get_object_handle::<CGenType>(typ)
            .context(anyhow!(
                "Member '{}' in struct '{}' has an unknown type '{}'",
                name,
                self.product.name,
                typ
            ))?;
        // done
        self.product
            .members
            .push(StructMember::new(name, ty_ref, array_len));
        Ok(self)
    }

    /// Build
    ///
    /// # Errors
    /// todo
    #[allow(clippy::unnecessary_wraps)]
    pub fn build(self) -> Result<StructType> {
        Ok(self.product)
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
        self.add_descriptor(name, array_len, DescriptorDef::Sampler)
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
            .context(anyhow!(
                "ConstantBuffer '{}' in DescriptorSet '{}' has an unknown type '{}'",
                name,
                self.product.name,
                inner_type
            ))?;
        self.add_descriptor(
            name,
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
            .context(anyhow!(
                "StructuredBuffer '{}' in DescriptorSet '{}' has an unknown type '{}'",
                name,
                self.product.name,
                inner_ty
            ))?;
        let def = StructuredBufferDef { ty_handle };
        let def = if read_write {
            DescriptorDef::RWStructuredBuffer(def)
        } else {
            DescriptorDef::StructuredBuffer(def)
        };
        self.add_descriptor(name, array_len, def)
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
        self.add_descriptor(name, array_len, def)
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
        array_len: Option<u32>,
        read_write: bool,
    ) -> Result<Self> {
        //
        // Texture format
        //
        let ty_handle = self
            .mdl
            .get_object_handle::<CGenType>(fmt)
            .context(anyhow!(
                "Texture '{}' in DescriptorSet '{}' has an unknown type '{}'",
                name,
                self.product.name,
                fmt
            ))?;
        let fmt_ty = ty_handle.get(self.mdl);
        let valid_type = match fmt_ty {
            CGenType::Struct(_) => false,
            CGenType::Native(e) => matches!(e, NativeType::Float(_)),
        };
        if !valid_type {
            return Err(anyhow!(
                "Format type '{}'for Texture '{}' in DescriptorSet '{}' is not valid",
                fmt,
                name,
                self.product.name
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
                        "Texture type '{}'for Texture '{}' in DescriptorSet '{}' cant be writable",
                        tex_type,
                        name,
                        self.product.name
                    ));
                }
                DescriptorDef::TextureCube(def)
            }
            "CubeArray" => {
                if read_write {
                    return Err(anyhow!(
                        "Texture type '{}'for Texture '{}' in DescriptorSet '{}' cant be writable",
                        tex_type,
                        name,
                        self.product.name
                    ));
                }
                DescriptorDef::TextureCubeArray(def)
            }
            _ => {
                return Err(anyhow!(
                    "Texture type '{}'for Texture '{}' in DescriptorSet '{}' is not valid",
                    tex_type,
                    name,
                    self.product.name
                ));
            }
        };

        self.add_descriptor(name, array_len, ds)
    }

    fn add_descriptor(
        mut self,
        name: &str,
        array_len: Option<u32>,
        def: DescriptorDef,
    ) -> Result<Self> {

        if let Some(array_len) = array_len {
            if array_len == 0 {
                return Err(anyhow!(
                    "Descriptor '{}' in DescriptorSet '{}' have array len set to 0",
                    name,
                    self.product.name
                ));    
            }
        }

        if self.names.contains(name) {
            return Err(anyhow!(
                "Descriptor '{}' in DescriptorSet '{}' already exists",
                name,
                self.product.name
            ));
        }
        self.names.insert(name.to_string());
        self.product.descriptors.push(Descriptor {
            name: name.to_owned(),
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

pub struct PipelineLayoutBuilder<'mdl> {
    mdl: &'mdl Model,
    product: PipelineLayout,
}

impl<'mdl> PipelineLayoutBuilder<'mdl> {
    pub fn new(mdl: &'mdl Model, name: &str) -> Self {
        PipelineLayoutBuilder {
            mdl,
            product: PipelineLayout::new(name),
        }
    }

    /// Add `DescriptorSet`.
    ///
    /// # Errors
    /// todo
    pub fn add_descriptor_set(mut self, ds_ty: &str) -> Result<Self> {
        // check descriptor_set exists
        let ds_handle = self.mdl.get_object_handle::<DescriptorSet>(ds_ty);
        if ds_handle.is_none() {
            return Err(anyhow!(
                "Unknown DescriptorSet '{}' added to PipelineLayout '{}'",
                ds_ty,
                self.product.name
            ));
        }
        let ds_handle = ds_handle.unwrap();
        let ds = ds_handle.get(self.mdl);

        // check for frequency conflict
        if self.product.descriptor_sets[ds.frequency as usize].is_some() {
            return Err(anyhow!(
                "Frequency conflict for DescriptorSet '{}' in PipelineLayout '{}'",
                ds_ty,
                self.product.name
            ));
        }
        self.product.descriptor_sets[ds.frequency as usize] = Some(ds_handle);

        Ok(self)
    }

    /// Add `PushConstant`.
    ///
    /// # Errors
    /// todo
    pub fn add_push_constant(mut self, typename: &str) -> Result<Self> {
        // only one push_constant is allowed
        if self.product.push_constant.is_some() {
            return Err(anyhow!(
                "Only one PushConstant allowed in PipelineLayout '{}'",
                self.product.name
            ));
        }
        // get cgen type and check its existence if necessary
        let ty_handle = self
            .mdl
            .get_object_handle::<CGenType>(typename)
            .context(anyhow!(
                "Unknown type '{}' for PushConstant in PipelineLayout '{}'",
                typename,
                self.product.name
            ))?;
        let cgen_type = ty_handle.get(self.mdl);
        // Only struct types allowed for now
        if let CGenType::Struct(_def) = cgen_type {
        } else {
            return Err(anyhow!("PushConstant must be Struct types "));
        }

        // done
        self.product.push_constant = Some(ty_handle);

        Ok(self)
    }

    /// build
    ///
    /// # Errors
    /// todo
    #[allow(clippy::unnecessary_wraps)]
    pub fn build(self) -> Result<PipelineLayout> {
        let mut first_none = None;

        for i in 0..self.product.descriptor_sets.len() {
            if self.product.descriptor_sets[i].is_none() && first_none.is_none() {
                first_none = Some(i);
            } else if self.product.descriptor_sets[i].is_some() && first_none.is_some() {
                return Err(anyhow!(
                    "DescriptorSets in PipelineLayout '{}' must be contiguous",
                    self.product.name
                ));
            }
        }

        Ok(self.product)
    }
}
