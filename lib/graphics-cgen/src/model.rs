use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use strum::*;

#[derive(Debug)]
pub struct Model {
    structs: ModelContainer<Struct>,
    descriptorsets: ModelContainer<DescriptorSet>,
    pipelinelayouts: ModelContainer<PipelineLayout>,
}

impl Model {
    pub fn new() -> Self {
        Model {
            structs: ModelContainer::new(),
            descriptorsets: ModelContainer::new(),
            pipelinelayouts: ModelContainer::new(),
        }
    }    

    pub fn add_struct(&mut self, def: Struct) -> Result<()> {
        self.structs.add(def.name.clone(), def)?;
        Ok(())
    }

    pub fn structs(&self) -> &ModelContainer<Struct> {
        &self.structs
    }

    pub fn add_descriptorset(&mut self, def: DescriptorSet) -> Result<()> {
        self.descriptorsets.add(def.name.clone(), def)?;
        Ok(())
    }

    pub fn descriptorsets(&self) -> &ModelContainer<DescriptorSet> {
        &self.descriptorsets
    }

    pub fn add_pipelinelayout(&mut self, def: PipelineLayout) -> Result<()> {
        self.pipelinelayouts.add(def.name.clone(), def)?;
        Ok(())
    }

    pub fn pipelinelayouts(&self) -> &ModelContainer<PipelineLayout> {
        &self.pipelinelayouts
    }

    pub fn try_get_type(&self, typename: &str) -> Option<CGenType> {
        let cgen_type = CGenType::from_str(typename).unwrap();

        if !self.contains_type(&cgen_type) {
            return None;
        }

        Some(cgen_type)
    }

    pub fn contains_type(&self, typ: &CGenType) -> bool {
        let result = if let CGenType::Complex(c) = typ {
            self.structs.contains(c)
        } else {
            true
        };

        result
    }

    pub fn get_struct_type_dependencies(&self, id: &str) -> Result<HashSet<CGenType>> {
        let s = self.structs.get(id)?;

        let result = s
            .members
            .iter()
            .filter_map(|m| {
                if let CGenType::Complex(_) = &m.cgen_type {
                    Some(m.cgen_type.clone())
                } else {
                    None
                }
            })
            .collect();

        Ok(result)
    }

    pub fn get_descriptorset_type_dependencies(&self, id: &str) -> Result<HashSet<CGenType>> {
        let mut result = HashSet::<CGenType>::new();

        let ds = self.descriptorsets.get(id)?;

        for d in &ds.descriptors {
            match &d.def {
                DescriptorDef::ConstantBuffer(def) => {
                    result.insert(def.inner_type.clone());
                    if let CGenType::Complex(t) = &def.inner_type {
                        result.extend(self.get_struct_type_dependencies(t)?);
                        // for x in t.drain() {
                        //     result.insert(x);
                        // }
                    }
                }
                DescriptorDef::StructuredBuffer(def) | DescriptorDef::RWStructuredBuffer(def) => {
                    result.insert(def.inner_type.clone());
                    if let CGenType::Complex(t) = &def.inner_type {
                        result.extend(self.get_struct_type_dependencies(t)?);
                    }
                }
                DescriptorDef::Sampler
                | DescriptorDef::ByteAddressBuffer
                | DescriptorDef::RWByteAddressBuffer
                | DescriptorDef::Texture2D(_)
                | DescriptorDef::RWTexture2D(_) => {}
            }
        }

        Ok(result)
    }

    pub fn get_pipelinelayout_type_dependencies(&self, id: &str) -> Result<HashSet<CGenType>> {
        let mut result = HashSet::<CGenType>::new();

        let pl = self.pipelinelayouts.get(id)?;

        for ds_name in pl.descriptorsets.iter() {
            result.extend(self.get_descriptorset_type_dependencies(&ds_name)?);
        }

        Ok(result)
    }
}

#[derive(Debug)]
struct ModelContainerInner<T> {
    objects: HashMap<String, T>,
}

#[derive(Debug)]
pub struct ModelContainer<T> {
    inner: Box<ModelContainerInner<T>>,
}

impl<T> ModelContainer<T> {
    pub fn new() -> Self {
        ModelContainer {
            inner: Box::new(ModelContainerInner{
                objects: HashMap::new(),
            })
        }
    }

    pub fn add(&mut self, id: String, entry: T) -> anyhow::Result<()> {
        if self.inner.objects.contains_key(&id) {
            return Err(anyhow!("Object '{}' already inserted.", id));
        }
        self.inner.objects.insert(id, entry);

        Ok(())
    }

    pub fn contains(&self, id: &str) -> bool {
        self.inner.objects.contains_key(id)
    }

    pub fn get(&self, id: &str) -> Result<&T> {
        match self.inner.objects.get(id) {
            Some(o) => Ok(o),
            None => Err(anyhow!("Unknown object '{}'", id)),
        }
    }

    pub fn try_get(&self, id: &str) -> Option<&T> {
        self.inner.objects.get(id)
    }

    pub fn iter(&self) -> std::collections::hash_map::Values<'_, String, T> {
        self.inner.objects.values()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CGenType {
    Float1,
    Float2,
    Float3,
    Float4,
    Complex(String),
}

impl FromStr for CGenType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s {
            "Float1" => CGenType::Float1,
            "Float2" => CGenType::Float2,
            "Float3" => CGenType::Float3,
            "Float4" => CGenType::Float4,
            _ => CGenType::Complex(s.to_owned()),
        };

        Ok(result)
    }
}
impl ToString for CGenType {
    fn to_string(&self) -> String {
        let type_str = match self {
            CGenType::Float1 => "Float1",
            CGenType::Float2 => "Float2",
            CGenType::Float3 => "Float3",
            CGenType::Float4 => "Float4",
            CGenType::Complex(t) => t,
        };
        type_str.to_owned()
    }
}

#[derive(Debug)]
pub struct StructMember {
    pub cgen_type: CGenType,
    pub name: String,
}

impl StructMember {
    pub fn new(name: &str, cgen_type: &CGenType) -> Self {
        StructMember {
            name: name.to_owned(),
            cgen_type: cgen_type.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Struct {
    pub name: String,
    pub members: Vec<StructMember>,
}

impl Struct {
    pub fn new(name: &str) -> Self {
        Struct {
            name: name.to_owned(),
            members: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, EnumString, AsRefStr)]
pub enum TextureFormat {
    R8,
    R8G8B8A8,
}

#[derive(Debug)]
pub struct TextureDef {
    pub inner_type: CGenType,
}

#[derive(Debug)]
pub struct SamplerDef {}

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

#[derive(Debug)]
pub struct ConstantBufferDef {
    pub inner_type: CGenType,
}

#[derive(Debug)]
pub struct StructuredBufferDef {
    pub inner_type: CGenType,
}

#[derive(Debug)]
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
}

#[derive(Debug)]
pub struct Descriptor {
    pub name: String,
    pub def: DescriptorDef,
}

#[derive(Debug)]
pub struct DescriptorSet {
    pub name: String,
    pub frequency: u32,
    pub descriptors: Vec<Descriptor>,
}

impl DescriptorSet {
    pub fn new(name: &str, frequency: u32) -> Self {
        DescriptorSet {
            name: name.to_owned(),
            frequency,
            descriptors: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct PushConstant {
    pub cgen_type: CGenType,
    pub name: String,
}

impl PushConstant {
    pub fn new(name: &str, cgen_type: &CGenType) -> Self {
        PushConstant {
            name: name.to_owned(),
            cgen_type: cgen_type.clone(),
        }
    }
}

#[derive(Debug)]
pub struct PipelineLayout {
    pub name: String,
    pub descriptorsets: Vec<String>,
    pub pushconstants: Vec<PushConstant>,
}

impl PipelineLayout {
    pub fn new(name: &str) -> PipelineLayout {
        PipelineLayout {
            name: name.to_owned(),
            descriptorsets: Vec::new(),
            pushconstants: Vec::new(),
        }
    }
}
