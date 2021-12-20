use lgn_graphics_api::{ShaderResourceType, MAX_DESCRIPTOR_SET_LAYOUTS};
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
        CGenVariant::Blob,
        RelativePath::new("cgen_def.blob").to_owned(),
        bincode::serialize(&cgen_def).unwrap(),
    )]
}

impl From<&CGenType> for CGenTypeDef {
    fn from(_cgen_type: &CGenType) -> Self {
        Self {}
    }
}

impl From<&DescriptorSet> for CGenDescriptorSetDef {
    fn from(descriptor_set: &DescriptorSet) -> Self {
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

                CGenDescriptorDef {
                    name: d.name.clone(),
                    array_size: d.array_len.unwrap_or(0),
                    shader_resource_type,
                }
            })
            .collect::<Vec<CGenDescriptorDef>>();

        Self {
            name: descriptor_set.name.clone(),
            frequency: descriptor_set.frequency,
            descriptor_defs,
        }
    }
}

impl From<&PipelineLayout> for CGenPipelineLayoutDef {
    fn from(pipeline_layout: &PipelineLayout) -> Self {
        let mut descriptor_set_layout_ids = [u32::MAX; MAX_DESCRIPTOR_SET_LAYOUTS];
        // pipeline_layout
        //     .descriptor_sets()
        //     .map(|x| (x.object_id(), x))
        //     .for_each(|(id, descriptor_set_info)| {
        //         descriptor_set_layout_ids[descriptor_set_info.frequency()] = id
        //     });

        Self {
            name: pipeline_layout.name.clone(),
            descriptor_set_layout_ids,
            push_constant_type: None,
        }
    }
}
