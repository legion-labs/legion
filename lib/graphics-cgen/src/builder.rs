use std::collections::HashSet;

use crate::model::{CGenType, ConstantBufferDef, Descriptor, DescriptorDef, DescriptorSet, Model, NativeType, PipelineLayout, PipelineLayoutContent, StructMember, StructType, StructuredBufferDef, TextureDef};
use anyhow::{anyhow, Context, Result};

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
        // let type_key = typ.into();
        let object_id = self.mdl.get_object_id::<CGenType>(typ).context(anyhow!(
            "Member '{}' in struct '{}' has an unknown type '{}'",
            name,
            self.product.name,
            typ
        ))?;
        // done
        self.product
            .members
            .push(StructMember::new(name, object_id, array_len));
        Ok(self)
    }

    pub fn build(self) -> Result<StructType> {
        Ok(self.product)
    }
}

pub struct DescriptorSetBuilder<'mdl> {
    mdl: &'mdl Model,
    product: DescriptorSet,
    names: HashSet<String>,
}

impl<'mdl> DescriptorSetBuilder<'mdl> {
    pub fn new(mdl: &'mdl Model, name: &str, frequency: u32) -> Self {
        DescriptorSetBuilder {
            mdl,
            product: DescriptorSet::new(name, frequency),
            names: HashSet::new(),
        }
    }

    pub fn add_samplers(self, name: &str, array_len: Option<u32>) -> Result<Self> {
        self.add_descriptor(name, array_len, DescriptorDef::Sampler)
    }

    pub fn add_constantbuffer(self, name: &str, inner_type: &str) -> Result<Self> {
        // get cgen type and check its existence if necessary
        let object_id = self
            .mdl
            .get_object_id::<CGenType>(inner_type)
            .context(anyhow!(
                "ConstantBuffer '{}' in DescriptorSet '{}' has an unknown type '{}'",
                name,
                self.product.name,
                inner_type
            ))?;
        let def = ConstantBufferDef { object_id };
        self.add_descriptor(name, None, DescriptorDef::ConstantBuffer(def))
    }

    pub fn add_structuredbuffer(
        self,
        name: &str,
        array_len: Option<u32>,
        inner_ty: &str,
        read_write: bool,
    ) -> Result<Self> {
        // get cgen type and check its existence if necessary
        let object_id = self
            .mdl
            .get_object_id::<CGenType>(inner_ty)
            .context(anyhow!(
                "StructuredBuffer '{}' in DescriptorSet '{}' has an unknown type '{}'",
                name,
                self.product.name,
                inner_ty
            ))?;
        let def = StructuredBufferDef { object_id };
        let def = if read_write {
            DescriptorDef::RWStructuredBuffer(def)
        } else {
            DescriptorDef::StructuredBuffer(def)
        };
        self.add_descriptor(name, array_len, def)
    }

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
        self.add_descriptor(name.clone(), array_len, def)
    }

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
        let ty_id = self
            .mdl
            .get_object_id::<CGenType>(fmt)
            .context(anyhow!(
                "Texture '{}' in DescriptorSet '{}' has an unknown type '{}'",
                name,
                self.product.name,
                fmt
            ))?;
        let fmt_ty = self.mdl.get_from_objectid::<CGenType>(ty_id).unwrap();
        let valid_type = {
            match fmt_ty {
                CGenType::Struct(_) => false,
                CGenType::Native(e) => match e {
                    NativeType::Float1
                    | NativeType::Float2
                    | NativeType::Float3
                    | NativeType::Float4 => true,
                    _ => false,
                },
            }
        };
        if !valid_type {
            return Err(anyhow!(
                "Format type '{}'for Texture '{}' in DescriptorSet '{}' is not valid",
                fmt,
                name,
                self.product.name
            ));
        }
        let def = TextureDef { object_id: ty_id };
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

        self.add_descriptor(name.clone(), array_len, ds)
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
            array_len,
            def,
        });

        Ok(self)
    }

    pub fn build(self) -> Result<DescriptorSet> {
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

    pub fn add_descriptorset(mut self, name: &str, ty: &str) -> Result<Self> {
        // check descriptorset exists
        let ds_id = self.mdl.get_object_id::<DescriptorSet>(ty);
        if ds_id.is_none() {
            return Err(anyhow!(
                "Unknown DescriptorSet '{}' added to PipelineLayout '{}'",
                ty,
                self.product.name
            ));
        }
        let obj_id = ds_id.unwrap();
        let ds = self.mdl.get_from_objectid::<DescriptorSet>(obj_id).unwrap();

        // check for frequency conflict
        if self.freqs.contains(&ds.frequency) {
            return Err(anyhow!(
                "Frequency conflict for DescriptorSet '{}' in PipelineLayout '{}'",
                ty,
                self.product.name
            ));
        }
        self.freqs.insert(ds.frequency);

        self.add_member(name, PipelineLayoutContent::DescriptorSet(obj_id))
    }

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
        let object_id = self
            .mdl
            .get_object_id::<CGenType>(typename)
            .context(anyhow!(
                "Unknown type '{}' for PushConstant '{}' in PipelineLayout '{}'",
                typename,
                name,
                self.product.name
            ))?;
        let cgen_type = self
            .mdl
            .get_from_objectid::<CGenType>(object_id)
            .context(anyhow!(
                "Unknown type '{}' for PushConstant '{}' in PipelineLayout '{}'",
                typename,
                name,
                self.product.name
            ))?;
        // Only struct types allowed for now
        if let CGenType::Struct(_def) = cgen_type {
        } else {
            return Err(anyhow!("PushConstant must be Struct types "));
        }
        // done
        self.add_member(name, PipelineLayoutContent::Pushconstant(object_id))
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

    pub fn build(self) -> Result<PipelineLayout> {
        Ok(self.product)
    }
}
