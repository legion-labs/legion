use lgn_graphics_api::ShaderResourceType;
use lgn_graphics_cgen_runtime::{
    CGenDef, CGenDescriptorDef, CGenDescriptorSetDef, CGenPipelineLayoutDef, CGenTypeDef,
};
use relative_path::RelativePath;

use crate::{
    generators::{product::Product, GeneratorContext},
    model::{CGenType, DescriptorSet, PipelineLayout},
    run::CGenVariant,
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut type_defs = Vec::with_capacity(ctx.model.size::<CGenType>());
    let mut descriptor_set_layout_defs = Vec::with_capacity(ctx.model.size::<DescriptorSet>());
    let mut root_signature_defs = Vec::with_capacity(ctx.model.size::<PipelineLayout>());

    for (_, cgen_type) in ctx.model.object_iter::<CGenType>().enumerate() {
        type_defs.push(cgen_type.into());
    }

    for (_, descriptor_set) in ctx.model.object_iter::<DescriptorSet>().enumerate() {
        descriptor_set_layout_defs.push(descriptor_set.into());
    }

    for (_, pipeline_layout) in ctx.model.object_iter::<PipelineLayout>().enumerate() {
        root_signature_defs.push(pipeline_layout.into());
    }

    let cgen_def = CGenDef {
        type_defs,
        descriptor_set_layout_defs,
        root_signature_defs,
    };

    vec![Product::new(
        CGenVariant::Rust,
        RelativePath::new("cgen_def.bin").to_owned(),
        bincode::serialize(&cgen_def).unwrap(),
    )]
}

impl From<&CGenType> for CGenTypeDef {
    fn from(cgen_type: &CGenType) -> Self {
        Self {}
    }
}

impl From<&DescriptorSet> for CGenDescriptorSetDef {
    fn from(descriptor_set: &DescriptorSet) -> Self {
        let mut flat_sampler_index = 0u32;
        let mut flat_buffer_index = 0u32;
        let mut flat_texture_index = 0u32;
        let descriptor_defs = descriptor_set
            .descriptors
            .iter()
            .map(|d| {
                let shader_resource_type = match d.def {
                    crate::model::DescriptorDef::Sampler => ShaderResourceType::Sampler,
                    crate::model::DescriptorDef::ConstantBuffer(_) => {
                        ShaderResourceType::ConstantBuffer
                    }
                    crate::model::DescriptorDef::StructuredBuffer(_) => {
                        ShaderResourceType::StructuredBuffer
                    }
                    crate::model::DescriptorDef::RWStructuredBuffer(_) => {
                        ShaderResourceType::RWStructuredBuffer
                    }
                    crate::model::DescriptorDef::ByteAddressBuffer => {
                        ShaderResourceType::ByteAdressBuffer
                    }
                    crate::model::DescriptorDef::RWByteAddressBuffer => {
                        ShaderResourceType::RWByteAdressBuffer
                    }
                    crate::model::DescriptorDef::Texture2D(_) => ShaderResourceType::Texture2D,
                    crate::model::DescriptorDef::RWTexture2D(_) => ShaderResourceType::RWTexture2D,
                    crate::model::DescriptorDef::Texture3D(_) => ShaderResourceType::Texture3D,
                    crate::model::DescriptorDef::RWTexture3D(_) => ShaderResourceType::RWTexture3D,
                    crate::model::DescriptorDef::Texture2DArray(_) => {
                        ShaderResourceType::Texture2DArray
                    }
                    crate::model::DescriptorDef::RWTexture2DArray(_) => {
                        ShaderResourceType::RWTexture2DArray
                    }
                    crate::model::DescriptorDef::TextureCube(_) => ShaderResourceType::TextureCube,
                    crate::model::DescriptorDef::TextureCubeArray(_) => {
                        ShaderResourceType::TextureCubeArray
                    }
                };

                let flat_index = match shader_resource_type {
                    ShaderResourceType::Sampler => flat_sampler_index,
                    ShaderResourceType::ConstantBuffer
                    | ShaderResourceType::StructuredBuffer
                    | ShaderResourceType::RWStructuredBuffer
                    | ShaderResourceType::ByteAdressBuffer
                    | ShaderResourceType::RWByteAdressBuffer => flat_buffer_index,
                    ShaderResourceType::Texture2D
                    | ShaderResourceType::RWTexture2D
                    | ShaderResourceType::Texture2DArray
                    | ShaderResourceType::RWTexture2DArray
                    | ShaderResourceType::Texture3D
                    | ShaderResourceType::RWTexture3D
                    | ShaderResourceType::TextureCube
                    | ShaderResourceType::TextureCubeArray => flat_texture_index,
                };

                let array_size = d.array_len.unwrap_or(0);

                match shader_resource_type {
                    ShaderResourceType::Sampler => flat_sampler_index += u32::max(1, array_size),
                    ShaderResourceType::ConstantBuffer
                    | ShaderResourceType::StructuredBuffer
                    | ShaderResourceType::RWStructuredBuffer
                    | ShaderResourceType::ByteAdressBuffer
                    | ShaderResourceType::RWByteAdressBuffer => {
                        flat_buffer_index += u32::max(1, array_size)
                    }
                    ShaderResourceType::Texture2D
                    | ShaderResourceType::RWTexture2D
                    | ShaderResourceType::Texture2DArray
                    | ShaderResourceType::RWTexture2DArray
                    | ShaderResourceType::Texture3D
                    | ShaderResourceType::RWTexture3D
                    | ShaderResourceType::TextureCube
                    | ShaderResourceType::TextureCubeArray => {
                        flat_texture_index += u32::max(1, array_size)
                    }
                };

                CGenDescriptorDef {
                    name: d.name.clone(),
                    array_size,
                    shader_resource_type,
                    flat_index,
                }
            })
            .collect::<Vec<CGenDescriptorDef>>();

        Self {
            name: descriptor_set.name.clone(),
            frequency: descriptor_set.frequency,
            descriptor_defs,
            flat_sampler_count: flat_sampler_index,
            flat_texture_count: flat_texture_index,
            flat_buffer_count: flat_buffer_index,
        }
    }
}

impl From<&PipelineLayout> for CGenPipelineLayoutDef {
    fn from(pipeline_layout: &PipelineLayout) -> Self {
        let descriptor_set_layout_ids = pipeline_layout
            .descriptor_sets()
            .map(|x| x.object_id())
            .collect();

        Self {
            name: pipeline_layout.name.clone(),
            descriptor_set_layout_ids,
            push_constant_type: None,
        }
    }
}
