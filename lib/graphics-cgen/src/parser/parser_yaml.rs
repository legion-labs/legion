use crate::model::*;
use crate::builder::*;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
struct YamlInclude {
    path : String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YamlStructMember {
    #[serde(rename = "type")]
    pub ty: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YamlStruct {
    pub name: String,
    pub members: Vec<YamlStructMember>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YamlDescriptor {
    pub name: String,
    #[serde(rename = "type")]
    pub descriptor_type: String,
    #[serde(rename = "inner_type")]
    #[serde(default)]
    pub cgen_type: String,    
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YamlDescriptorSet {
    pub name: String,
    pub frequency: u32,
    pub descriptors: Vec<YamlDescriptor>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YamlPushConstant {
    pub name: String,
    #[serde(rename = "type")]
    pub cgen_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct YamlPipelineLayout {
    pub name: String,
    pub descriptorssets: Vec<String>,
    pub pushconstants: Vec<YamlPushConstant>,
}

#[derive(Debug, Deserialize, Serialize)]
enum YamlPrimitive {    
    Include(YamlInclude),
    Struct(YamlStruct),
    DescriptorSet(YamlDescriptorSet),
    PipelineLayout(YamlPipelineLayout),
}

#[derive(Debug, Deserialize, Serialize)]
struct YamlModel ( Option<Vec<YamlPrimitive>> );


pub fn from_yaml(file_path: &Path) -> anyhow::Result<Arc<Model>> {

    let mut model = Model::new();    

    process_yaml_model(&mut model, &file_path)?;

    Ok(Arc::new(model))
}

fn process_yaml_model(model: &mut Model, file_path: &Path) -> anyhow::Result<()> {    

    assert!(file_path.is_absolute());
    
    let file_folder = file_path.parent().unwrap();    
    let yaml_model = load_yaml_file(file_path)?;    
    
    if let Some(p) = &yaml_model.0 {

        for prim in p {
    
            match prim {            
    
                YamlPrimitive::Include(yaml_inc) => {
                    process_yaml_include(model, file_folder, yaml_inc).context(format!( "Cannot include file '{}' from {}", yaml_inc.path, file_path.display()))?;
                }
    
                YamlPrimitive::Struct(yaml_struct) => {
                    process_yaml_struct(model, yaml_struct).context(format!( "Cannot add Struct '{}' from '{}'", &yaml_struct.name, file_path.display()))?;
                }
    
                YamlPrimitive::DescriptorSet(yaml_ds) => {
                    process_yaml_descriptorset(model, yaml_ds).context(format!( "Cannot add DescriptorSet '{}' from '{}'", &yaml_ds.name, file_path.display()))?;
                }
    
                YamlPrimitive::PipelineLayout(yaml_pl) => {
                    process_yaml_pipelinelayout(model, yaml_pl).context(format!( "Cannot add PipelineLayout '{}' from '{}'", &yaml_pl.name, file_path.display()))?;
                }            
            }
        }
    }

    Ok(())
}

fn process_yaml_include(model : &mut Model, file_folder :&Path, yaml_inc : &YamlInclude) -> anyhow::Result<()> {    

    let mut inc_path = PathBuf::from_str(&yaml_inc.path)?;        
    if inc_path.is_relative() {
        let mut abs_path = PathBuf::from(file_folder);
        abs_path.push(inc_path);                                        
        inc_path = abs_path;        
    }    

    process_yaml_model(model, &inc_path )?;

    Ok(())
}

fn process_yaml_struct(model : &mut Model, yaml_struct : &YamlStruct) -> anyhow::Result<()> {    
    
    let mut builder = StructBuilder::new(model, &yaml_struct.name);
    for mb in &yaml_struct.members {
        builder = builder
            .add_member(&mb.name, &mb.ty)?;
    }
    let product = builder.build()?;

    model.add( CGenType::Struct(product))?;

    Ok(())
}

fn process_yaml_descriptorset(model : &mut Model, yaml_ds : &YamlDescriptorSet) -> anyhow::Result<()> {    

    let mut builder = DescriptorSetBuilder::new(
        model,
        &yaml_ds.name,
        yaml_ds.frequency,
    );
    for ds in &yaml_ds.descriptors {
        let descriptor_type =
            DescriptorType::from_str(&ds.descriptor_type).context( format!("Unknown descriptor type '{}'", ds.descriptor_type))?;
        match descriptor_type {
            DescriptorType::Sampler => {
                builder = builder.add_sampler(&ds.name)?
            }
            DescriptorType::ConstantBuffer => {
                builder = builder.add_constantbuffer(&ds.name, &ds.cgen_type)?
            }
            DescriptorType::StructuredBuffer => {
                builder = builder.add_structuredbuffer(&ds.name, &ds.cgen_type)?
            }
            DescriptorType::RWStructuredBuffer => {
                builder = builder.add_rwstructuredbuffer(&ds.name,&ds.cgen_type)?
            }
            DescriptorType::ByteAddressBuffer => {
                builder = builder.add_byteaddressbuffer(&ds.name)?
            }
            DescriptorType::RWByteAddressBuffer => {
                builder = builder.add_rwbyteaddressbuffer(&ds.name)?
            }
            DescriptorType::Texture2D => {                
                builder = builder.add_texture2d(&ds.name, &ds.cgen_type)?
            }
            DescriptorType::RWTexture2D => {
                builder = builder.add_rwtexture2d(&ds.name, &ds.cgen_type)?
            }
        };
    }

    let product = builder.build()?;
    model.add( product)?;

    Ok(())
}

fn process_yaml_pipelinelayout(model : &mut Model, yaml_pl : &YamlPipelineLayout) -> anyhow::Result<()> {    
    
    let mut builder = PipelineLayoutBuilder::new(model, &yaml_pl.name);
    for ds in &yaml_pl.descriptorssets {
        builder = builder.add_descriptorset(ds)?;
    }
    for pc in &yaml_pl.pushconstants {
        builder =
            builder.add_pushconstant(&pc.name, &pc.cgen_type)?;
    }

    let product = builder.build()?;
    model.add(product)?;

    Ok(())
}

fn load_yaml_file(file_path: &Path) -> Result<YamlModel> {

    if !file_path.is_file() {
        return Err(anyhow!("Invalid file path {}", file_path.display()));    
    }
    let file_content = std::fs::read_to_string(&file_path)
        .context(format!("Failed to load {}", file_path.display()))?;        
    let result: YamlModel = serde_yaml::from_str(&file_content).context(format!("Failed to parse {}", file_path.display()))?;
    Ok(result)
}
