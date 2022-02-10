use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Context, Result};
use rune::{
    ast::Span,
    compile::{CompileError, FileSourceLoader, Item, SourceLoader},
    termcolor::{ColorChoice, StandardStream},
    Diagnostics, Source, Sources, Vm,
};
use serde::{Deserialize, Serialize};

use super::ParsingResult;
use crate::db::{
    self, CGenType, DescriptorSetBuilder, PipelineLayoutBuilder, ShaderBuilder, StructBuilder,
};

pub(crate) fn from_rune(file_path: &Path) -> Result<ParsingResult> {
    assert!(file_path.is_absolute());

    from_rune_internal(file_path)
        .with_context(|| anyhow!("When running rune script '{}'", file_path.display()))
}

fn from_rune_internal(file_path: &Path) -> Result<ParsingResult> {
    // Store dependencies
    let mut input_dependencies = vec![file_path.to_owned()];

    // Initialize input source
    let mut sources = {
        let source = Source::from_path(file_path)?;
        let mut sources = Sources::new();
        sources.insert(source);
        sources
    };

    // Initialize context
    let context = rune::Context::with_default_modules()?;

    // Compile script
    let unit = {
        let mut diagnostics = Diagnostics::new();
        let mut source_loader = RuneSourceLoader::default();
        let result = rune::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .with_source_loader(&mut source_loader)
            .build();

        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            diagnostics.emit(&mut writer, &sources)?;
        }

        input_dependencies.extend(source_loader.dependencies);

        result?
    };

    // Initialize VM and run
    let output = {
        let mut vm = Vm::new(Arc::new(context.runtime()), Arc::new(unit));

        vm.call(&["main"], ())?
    };

    // Transform to JSON and create object list
    let object_list = {
        let str = serde_json::to_string(&output)?;

        let object_list: Vec<CGenObj> = serde_json::from_str(&str)?;

        object_list
    };

    // Create model
    let model = {
        let mut model = db::create();
        for object in &object_list {
            match object {
                CGenObj::Struct(data) => {
                    add_struct(&mut model, data)?;
                }
                CGenObj::DescriptorSet(data) => {
                    add_descriptor_set(&mut model, data)?;
                }
                CGenObj::PipelineLayout(data) => {
                    add_pipeline_layout(&mut model, data)?;
                }
                CGenObj::Shader(data) => {
                    add_shader(&mut model, data)?;
                }
            }
        }
        model
    };

    // Result
    Ok(ParsingResult {
        input_dependencies,
        model,
    })
}

fn add_struct(model: &mut db::Model, data: &StructData) -> Result<()> {
    add_struct_internal(model, data).with_context(|| anyhow!("When parsing struct '{}'", data.name))
}

fn add_struct_internal(model: &mut db::Model, data: &StructData) -> Result<()> {
    let mut builder = StructBuilder::new(&*model, &data.name);
    for f in &data.fields {
        builder = builder.add_member(&f.name, &f.ty, f.array_len)?;
    }
    let struct_type = builder.build()?;
    model.add(&data.name, CGenType::Struct(struct_type))?;
    Ok(())
}

fn add_descriptor_set(model: &mut db::Model, data: &DescriptorSetData) -> Result<()> {
    add_descriptor_set_internal(model, data)
        .with_context(|| anyhow!("When building descriptor set '{}'", data.name))
}

fn add_descriptor_set_internal(model: &mut db::Model, data: &DescriptorSetData) -> Result<()> {
    let mut builder = DescriptorSetBuilder::new(&*model, &data.name, data.frequency);
    for descriptor in &data.descriptors {
        match &descriptor {
            DescriptorData::ConstantBuffer(def) => {
                builder = builder.add_constant_buffer(&def.name, &def.content)?;
            }
            DescriptorData::StructuredBuffer(def) | DescriptorData::RWStructuredBuffer(def) => {
                builder = builder.add_structured_buffer(
                    &def.name,
                    def.array_len,
                    &def.content,
                    descriptor.read_write(),
                )?;
            }
            DescriptorData::ByteAddressBuffer(def) | DescriptorData::RWByteAddressBuffer(def) => {
                builder = builder.add_byte_address_buffer(
                    &def.name,
                    def.array_len,
                    descriptor.read_write(),
                )?;
            }
            DescriptorData::Texture2D(def) | DescriptorData::RWTexture2D(def) => {
                builder = builder.add_texture(
                    &def.name,
                    "2D",
                    &def.content,
                    def.array_len,
                    descriptor.read_write(),
                )?;
            }
            DescriptorData::Texture3D(def) | DescriptorData::RWTexture3D(def) => {
                builder = builder.add_texture(
                    &def.name,
                    "3D",
                    &def.content,
                    def.array_len,
                    descriptor.read_write(),
                )?;
            }
            DescriptorData::Texture2DArray(def) | DescriptorData::RWTexture2DArray(def) => {
                builder = builder.add_texture(
                    &def.name,
                    "2DArray",
                    &def.content,
                    def.array_len,
                    descriptor.read_write(),
                )?;
            }
            DescriptorData::TextureCube(def) => {
                builder =
                    builder.add_texture(&def.name, "Cube", &def.content, def.array_len, false)?;
            }
            DescriptorData::TextureCubeArray(def) => {
                builder = builder.add_texture(
                    &def.name,
                    "CubeArray",
                    &def.content,
                    def.array_len,
                    false,
                )?;
            }
            DescriptorData::Sampler(def) => {
                builder = builder.add_samplers(&def.name, def.array_len)?;
            }
        }
    }
    let descriptor_set = builder.build()?;
    model.add(&data.name, descriptor_set)?;
    Ok(())
}

fn add_pipeline_layout(model: &mut db::Model, data: &PipelineLayoutData) -> Result<()> {
    add_pipeline_layout_internal(model, data)
        .with_context(|| anyhow!("When building pipeline layout '{}'", data.name))
}

fn add_pipeline_layout_internal(model: &mut db::Model, data: &PipelineLayoutData) -> Result<()> {
    let mut builder = PipelineLayoutBuilder::new(&*model, &data.name);
    for descriptor_set in &data.descriptor_sets {
        builder = builder.add_descriptor_set(descriptor_set)?;
    }
    if let Some(push_constant) = &data.push_constant {
        builder = builder.add_push_constant(push_constant)?;
    }
    let pipeline_layout = builder.build()?;
    model.add(&data.name, pipeline_layout)?;
    Ok(())
}

fn add_shader(model: &mut db::Model, data: &ShaderData) -> Result<()> {
    add_shader_internal(model, data)
        .with_context(|| anyhow!("When building shader '{}'", data.name))
}

fn add_shader_internal(model: &mut db::Model, data: &ShaderData) -> Result<()> {
    let mut builder = ShaderBuilder::new(&*model, &data.name);

    builder = builder.set_path(&data.path)?;
    for stage in &data.stages {
        builder = builder.add_stage(stage)?;
    }
    builder = builder.set_pipeline_layout(&data.pipeline_layout)?;
    for define in &data.defines {
        builder = builder.add_define(define)?;
    }
    let shader = builder.build()?;
    model.add(&data.name, shader)?;
    Ok(())
}

//
// RuneSourceLoader
//
#[derive(Default)]
struct RuneSourceLoader {
    default_loader: FileSourceLoader,
    dependencies: Vec<PathBuf>,
}

impl SourceLoader for RuneSourceLoader {
    fn load(&mut self, root: &Path, item: &Item, span: Span) -> Result<Source, CompileError> {
        let result = self.default_loader.load(root, item, span);

        if let Ok(source) = &result {
            self.dependencies.push(source.path().unwrap().to_owned());
        }

        result
    }
}

//
// CGenObj
//
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum CGenObj {
    // Constant(ConstantData),
    Struct(StructData),
    DescriptorSet(DescriptorSetData),
    PipelineLayout(PipelineLayoutData),
    Shader(ShaderData),
}

//
// Struct
//
#[derive(Debug, Serialize, Deserialize)]
struct MemberData {
    name: String,
    ty: String,
    array_len: Option<u32>,
    value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StructData {
    name: String,
    fields: Vec<MemberData>,
}

//
// ConstBuffer
//
#[derive(Serialize, Deserialize, Debug)]
struct ConstBufferData {
    name: String,
    content: String,
}

//
// StructuredBuffer
//
#[derive(Serialize, Deserialize, Debug)]
struct StructuredBufferData {
    name: String,
    content: String,
    array_len: Option<u32>,
}

//
// ByteAddressBuffer
//
#[derive(Serialize, Deserialize, Debug)]
struct ByteAddressBufferData {
    name: String,
    array_len: Option<u32>,
}

//
// Texture
//
#[derive(Serialize, Deserialize, Debug)]
struct TextureData {
    name: String,
    content: String,
    array_len: Option<u32>,
}

//
// Sampler
//
#[derive(Serialize, Deserialize, Debug)]
struct SamplerData {
    name: String,
    array_len: Option<u32>,
}

//
// DescriptorSet
//
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum DescriptorData {
    ConstantBuffer(ConstBufferData),
    StructuredBuffer(StructuredBufferData),
    RWStructuredBuffer(StructuredBufferData),
    ByteAddressBuffer(ByteAddressBufferData),
    RWByteAddressBuffer(ByteAddressBufferData),
    Texture2D(TextureData),
    RWTexture2D(TextureData),
    Texture3D(TextureData),
    RWTexture3D(TextureData),
    Texture2DArray(TextureData),
    RWTexture2DArray(TextureData),
    TextureCube(TextureData),
    TextureCubeArray(TextureData),
    Sampler(SamplerData),
}

impl DescriptorData {
    fn read_write(&self) -> bool {
        match self {
            DescriptorData::RWStructuredBuffer(_)
            | DescriptorData::RWByteAddressBuffer(_)
            | DescriptorData::RWTexture2D(_)
            | DescriptorData::RWTexture3D(_)
            | DescriptorData::RWTexture2DArray(_) => true,
            DescriptorData::ConstantBuffer(_)
            | DescriptorData::StructuredBuffer(_)
            | DescriptorData::ByteAddressBuffer(_)
            | DescriptorData::Texture2D(_)
            | DescriptorData::Texture3D(_)
            | DescriptorData::Texture2DArray(_)
            | DescriptorData::TextureCube(_)
            | DescriptorData::TextureCubeArray(_)
            | DescriptorData::Sampler(_) => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DescriptorSetData {
    name: String,
    frequency: u32,
    descriptors: Vec<DescriptorData>,
}

//
// PipelineLayout
//
#[derive(Serialize, Deserialize, Debug)]
struct PipelineLayoutData {
    name: String,
    #[serde(default)]
    descriptor_sets: Vec<String>,
    push_constant: Option<String>,
}

//
// Shader
//
#[derive(Serialize, Deserialize, Debug)]
struct ShaderData {
    name: String,
    path: String,
    stages: Vec<String>,
    pipeline_layout: String,
    #[serde(default)]
    defines: Vec<String>,
}
