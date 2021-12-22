use std::collections::HashSet;

use anyhow::{anyhow, Context, Result};

use crate::model::{
    CGenType, ConstantBufferDef, Descriptor, DescriptorDef, DescriptorSet, Model, NativeType,
    PipelineLayout, PipelineLayoutContent, StructMember, StructType, StructuredBufferDef,
    TextureDef,
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

        // get cgen type and check its existence if necessary
        let ty_ref = self.mdl.get_object_ref::<CGenType>(typ).context(anyhow!(
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
            flat_index: 0
        }
    }

    /// Add descriptor.
    ///
    /// # Errors
    /// todo
    pub fn add_samplers(self, name: &str, array_len: Option<u32>) -> Result<Self> {
        self.add_descriptor(name, array_len, DescriptorDef::Sampler)
    }

    /// Add descriptor.
    ///
    /// # Errors
    /// todo
    pub fn add_constantbuffer(self, name: &str, inner_type: &str) -> Result<Self> {
        // get cgen type and check its existence if necessary
        let ty_ref = self
            .mdl
            .get_object_ref::<CGenType>(inner_type)
            .context(anyhow!(
                "ConstantBuffer '{}' in DescriptorSet '{}' has an unknown type '{}'",
                name,
                self.product.name,
                inner_type
            ))?;
        self.add_descriptor(
            name,
            None,
            DescriptorDef::ConstantBuffer(ConstantBufferDef { ty_ref }),
        )
    }

    /// Add descriptor.
    ///
    /// # Errors
    /// todo
    pub fn add_structuredbuffer(
        self,
        name: &str,
        array_len: Option<u32>,
        inner_ty: &str,
        read_write: bool,
    ) -> Result<Self> {
        // get cgen type and check its existence if necessary
        let ty_ref = self
            .mdl
            .get_object_ref::<CGenType>(inner_ty)
            .context(anyhow!(
                "StructuredBuffer '{}' in DescriptorSet '{}' has an unknown type '{}'",
                name,
                self.product.name,
                inner_ty
            ))?;
        let def = StructuredBufferDef { ty_ref };
        let def = if read_write {
            DescriptorDef::RWStructuredBuffer(def)
        } else {
            DescriptorDef::StructuredBuffer(def)
        };
        self.add_descriptor(name, array_len, def)
    }

    /// Add descriptor.
    ///
    /// # Errors
    /// todo
    pub fn add_byteaddressbuffer(
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
        let ty_ref = self.mdl.get_object_ref::<CGenType>(fmt).context(anyhow!(
            "Texture '{}' in DescriptorSet '{}' has an unknown type '{}'",
            name,
            self.product.name,
            fmt
        ))?;
        let fmt_ty = ty_ref.get(self.mdl);
        // let fmt_ty = self.mdl.get_from_objectid::<CGenType>(ty_ref).unwrap();
        let valid_type = match fmt_ty {
            CGenType::Struct(_) => false,
            CGenType::Native(e) => matches!(
                e,
                NativeType::Float1 | NativeType::Float2 | NativeType::Float3 | NativeType::Float4
            ),
        };
        if !valid_type {
            return Err(anyhow!(
                "Format type '{}'for Texture '{}' in DescriptorSet '{}' is not valid",
                fmt,
                name,
                self.product.name
            ));
        }
        let def = TextureDef { ty_ref };
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
    pub fn build(mut self) -> Result<DescriptorSet> {
        self.product.flat_descriptor_count = self.flat_index;
        Ok(self.product)
    }
}

pub struct PipelineLayoutBuilder<'mdl> {
    mdl: &'mdl Model,
    product: PipelineLayout,
    names: HashSet<String>,
    freqs: HashSet<u32>,
    has_pushconstant: bool,
}

impl<'mdl> PipelineLayoutBuilder<'mdl> {
    pub fn new(mdl: &'mdl Model, name: &str) -> Self {
        PipelineLayoutBuilder {
            mdl,
            product: PipelineLayout::new(name),
            names: HashSet::new(),
            freqs: HashSet::new(),
            has_pushconstant: false,
        }
    }

    /// Add descriptorset.
    ///
    /// # Errors
    /// todo
    pub fn add_descriptorset(mut self, name: &str, ty: &str) -> Result<Self> {
        // check descriptorset exists
        let ds_ref = self.mdl.get_object_ref::<DescriptorSet>(ty);
        if ds_ref.is_none() {
            return Err(anyhow!(
                "Unknown DescriptorSet '{}' added to PipelineLayout '{}'",
                ty,
                self.product.name
            ));
        }
        let ds_ref = ds_ref.unwrap();
        let ds = ds_ref.get(self.mdl);

        // check for frequency conflict
        if self.freqs.contains(&ds.frequency) {
            return Err(anyhow!(
                "Frequency conflict for DescriptorSet '{}' in PipelineLayout '{}'",
                ty,
                self.product.name
            ));
        }
        self.freqs.insert(ds.frequency);

        self.add_member(name, PipelineLayoutContent::DescriptorSet(ds_ref))
    }

    /// Add pushconstant
    ///
    /// # Errors
    /// todo
    pub fn add_pushconstant(mut self, name: &str, typename: &str) -> Result<Self> {
        // only one pushconstant is allowed
        if self.has_pushconstant {
            return Err(anyhow!(
                "Only one PushConstant allowed in PipelineLayout '{}'",
                self.product.name
            ));
        }
        self.has_pushconstant = true;
        // get cgen type and check its existence if necessary
        let ty_ref = self
            .mdl
            .get_object_ref::<CGenType>(typename)
            .context(anyhow!(
                "Unknown type '{}' for PushConstant '{}' in PipelineLayout '{}'",
                typename,
                name,
                self.product.name
            ))?;
        let cgen_type = ty_ref.get(self.mdl);
        // Only struct types allowed for now
        if let CGenType::Struct(_def) = cgen_type {
        } else {
            return Err(anyhow!("PushConstant must be Struct types "));
        }
        // done
        self.add_member(name, PipelineLayoutContent::Pushconstant(ty_ref))
    }

    fn add_member(mut self, name: &str, mb: PipelineLayoutContent) -> Result<Self> {
        if self.names.contains(name) {
            return Err(anyhow!(
                "Member '{}' in PipelineLayout '{}' already exists",
                name,
                self.product.name
            ));
        }
        self.names.insert(name.to_string());
        self.product.members.push((name.to_string(), mb));

        Ok(self)
    }

    /// build
    ///
    /// # Errors
    /// todo
    pub fn build(self) -> Result<PipelineLayout> {
        Ok(self.product)
    }
}
