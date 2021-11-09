use crate::model::*;
use anyhow::{anyhow, Context, Result};

pub struct StructBuilder<'mdl> {
    mdl: &'mdl Model,
    product: Struct,
}

impl<'mdl> StructBuilder<'mdl> {

    pub fn new(mdl: &'mdl Model, name: &str) -> Self {        
        StructBuilder {
            mdl,
            product: Struct::new(name),
        }
    }

    pub fn add_member(mut self, name: &str, typename: &str) -> Result<Self> {
        // check member uniqueness
        if self
            .product
            .members
            .iter()
            .position(|e| &e.name == name)
            .is_some()
        {
            return Err(anyhow!(
                "Member '{}' already exists in struct '{}'",
                name,
                self.product.name
            ));
        }
        // get cgen type and check its existence if necessary
        let cgen_type = self.mdl.try_get_type(typename).context(anyhow!(format!(
            "Member '{}' in struct '{}' has an unknown type '{}'",
            name, self.product.name, typename
        )))?;
        // done
        self.product
            .members
            .push(StructMember::new(name, &cgen_type));
        Ok(self)
    }

    pub fn build(self) -> Result<Struct> {

        Ok(self.product)
    }
}

pub struct DescriptorSetBuilder<'mdl> {
    mdl: &'mdl Model,
    product: DescriptorSet,
}

impl<'mdl> DescriptorSetBuilder<'mdl> {
    pub fn new(mdl: &'mdl Model, name: &str, frequency: u32) -> Self {
        DescriptorSetBuilder {
            mdl,
            product: DescriptorSet::new(name, frequency),
        }
    }

    pub fn add_sampler(self, name: &str) -> Result<Self> {
        let def = DescriptorDef::Sampler;
        self.add_descriptor(name, def)
    }

    pub fn add_constantbuffer(self, name: &str, typename: &str) -> Result<Self> {
        // get cgen type and check its existence if necessary
        let inner_type = self.mdl.try_get_type(typename).context(anyhow!(format!(
            "ConstantBuffer '{}' in DescriptorSet '{}' has an unknown type '{}'",
            name, self.product.name, typename
        )))?;
        let def = ConstantBufferDef {
            inner_type: inner_type,
        };
        let def = DescriptorDef::ConstantBuffer(def);
        self.add_descriptor(name, def)
    }

    pub fn add_structuredbuffer(self, name: &str, typename: &str) -> Result<Self> {
        self.add_structuredbuffer_internal(name, typename, false)
    }

    pub fn add_rwstructuredbuffer(self, name: &str, typename: &str) -> Result<Self> {
        self.add_structuredbuffer_internal(name, typename, true)
    }

    pub fn add_byteaddressbuffer(self, name: &str) -> Result<Self> {
        self.add_byteaddressbuffer_internal(name, false)
    }

    pub fn add_rwbyteaddressbuffer(self, name: &str) -> Result<Self> {
        self.add_byteaddressbuffer_internal(name, true)
    }

    pub fn add_texture2d(self, name: &str, format: &str) -> Result<Self> {
        self.add_texture2d_internal(name, format, false)
    }

    pub fn add_rwtexture2d(self, name: &str, format: &str) -> Result<Self> {
        self.add_texture2d_internal(name, format, true)
    }

    fn add_structuredbuffer_internal(self, name: &str, typename: &str, uav: bool) -> Result<Self> {
        // get cgen type and check its existence if necessary
        let inner_type = self.mdl.try_get_type(typename).context(anyhow!(format!(
            "StructuredBuffer '{}' in DescriptorSet '{}' has an unknown type '{}'",
            name, self.product.name, typename
        )))?;
        let def = StructuredBufferDef { inner_type };
        let def = if uav {
            DescriptorDef::RWStructuredBuffer(def)
        } else {
            DescriptorDef::StructuredBuffer(def)
        };
        self.add_descriptor(name, def)
    }

    fn add_byteaddressbuffer_internal(self, name: &str, uav: bool) -> Result<Self> {
        let def = if uav {
            DescriptorDef::RWByteAddressBuffer
        } else {
            DescriptorDef::ByteAddressBuffer
        };
        self.add_descriptor(name.clone(), def)
    }

    pub fn add_texture2d_internal(self, name: &str, typename: &str, uav: bool) -> Result<Self> {
        let inner_type = self.mdl.try_get_type(typename).context(anyhow!(format!(
            "Texture '{}' in DescriptorSet '{}' has an unknown type '{}'",
            name, self.product.name, typename
        )))?;
        match inner_type {
            CGenType::Complex(_) => {
                return Err(anyhow!(format!(
                    "Inner type '{}'for Texture '{}' in DescriptorSet '{}' is not valid",
                    typename, name, self.product.name
                )))
            }
            CGenType::Float1 | CGenType::Float2 | CGenType::Float3 | CGenType::Float4 => {}
        };        
        let def = TextureDef { inner_type };
        let ds = if uav {
            DescriptorDef::RWTexture2D(def)
        } else {
            DescriptorDef::Texture2D(def)
        };
        self.add_descriptor(name.clone(), ds)
    }

    fn add_descriptor(mut self, name: &str, def: DescriptorDef) -> Result<Self> {
        // check descriptor uniqueness
        if self
            .product
            .descriptors
            .iter()
            .position(|e| e.name == name)
            .is_some()
        {
            return Err(anyhow!(
                "Descriptor '{}' in DescriptorSet '{}' already exists",
                name,
                self.product.name
            ));
        }
        // add descriptor
        self.product.descriptors.push(Descriptor {
            name: name.to_owned(),
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
}

impl<'mdl> PipelineLayoutBuilder<'mdl> {
    pub fn new(mdl: &'mdl Model, name: &str) -> Self {
        PipelineLayoutBuilder {
            mdl,
            product: PipelineLayout::new(name),
        }
    }

    pub fn add_descriptorset(mut self, name: &str) -> Result<Self> {
        // check descriptorset exists
        let ds = self.mdl.descriptorsets().try_get(name);
        if ds.is_none() {
            return Err(anyhow!(
                "Unknown DescriptorSet '{}' added to PipelineLayout '{}'",
                name,
                self.product.name
            ));
        }
        let ds = ds.unwrap();
        // check descriptorset uniqueness
        if self
            .product
            .descriptorsets
            .iter()
            .position(|e| e == name)
            .is_some()
        {
            return Err(anyhow!(
                "DescriptorSet '{}' in PipelineLayout '{}' already exists",
                name,
                self.product.name
            ));
        }
        // check for frequency conflict
        if self
            .product
            .descriptorsets
            .iter()
            .filter(|e| self.mdl.descriptorsets().get(e).unwrap().frequency == ds.frequency)
            .count()
            > 0
        {
            return Err(anyhow!(
                "Frequency conflict for DescriptorSet '{}' in PipelineLayout '{}'",
                name,
                self.product.name
            ));
        }
        // done
        self.product.descriptorsets.push(name.to_owned());
        Ok(self)
    }

    pub fn add_pushconstant(mut self, name: &str, cgen_type: &str) -> Result<Self> {
        // check member uniqueness
        if self
            .product
            .pushconstants
            .iter()
            .position(|e| &e.name == name)
            .is_some()
        {
            return Err(anyhow!(
                "PushConstant '{}' in PipelineLayout '{}' already exists",
                name,
                self.product.name
            ));
        }
        // get cgen type and check its existence if necessary
        let typ = self.mdl.try_get_type(cgen_type).context(anyhow!(format!(
            "Unknown type '{}' for PushConstant '{}' in PipelineLayout '{}'",
            cgen_type, name, self.product.name
        )))?;
        // done
        self.product
            .pushconstants
            .push(PushConstant::new(name, &typ));
        Ok(self)
    }

    pub fn build(mut self) -> Result<PipelineLayout> {
        // collect descriptorsets
        let mut descriptorsets : Vec<_> = 
            self.product.descriptorsets
            .iter()
            .map(|ds_name| self.mdl.descriptorsets().get(ds_name).unwrap() )
            .collect();
        // check descriptors uniqueness
        let mut all_descriptor_names: Vec<&str> = Vec::new();
        for ds in &descriptorsets {
            for d in &ds.descriptors {                
                if all_descriptor_names.iter().find(|x| d.name == **x ).is_some() {
                    return Err( anyhow!(format!("Many Descriptors named '{}' detected in PipelineLayout '{}'", d.name, self.product.name)) );
                }
                all_descriptor_names.push(d.name.as_str());
            }
        }

        // sort by frequency        
        descriptorsets.sort_by(|a, b| a.frequency.cmp(&b.frequency) );
        self.product.descriptorsets = descriptorsets.iter().map( |ds| ds.name.clone() ).collect();

        Ok(self.product)
    }
}
