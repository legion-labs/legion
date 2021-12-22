use lgn_graphics_api::{ShaderResourceType, MAX_DESCRIPTOR_SET_LAYOUTS};
use lgn_graphics_cgen_runtime::{
    CGenDef, CGenDescriptorDef, CGenDescriptorSetDef, CGenPipelineLayoutDef, CGenTypeDef,
    CGenTypeId,
};
use relative_path::RelativePath;

use crate::{
    generators::{product::Product, GeneratorContext},
    model::{
        CGenType, CGenTypeRef, DescriptorSet, DescriptorSetRef, Model, PipelineLayout,
        PipelineLayoutRef,
    },
    run::CGenVariant,
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    /*    let mut type_defs = Vec::with_capacity(ctx.model.size::<CGenType>());
        let mut descriptor_set_layout_defs = Vec::with_capacity(ctx.model.size::<DescriptorSet>());
        let mut root_signature_defs = Vec::with_capacity(ctx.model.size::<PipelineLayout>());

        for (_, cgen_type) in ctx.model.ref_iter::<CGenType>().enumerate() {
            type_defs.push(into_cgen_type(ctx.model, cgen_type));
        }

        for (_, descriptor_set) in ctx.model.ref_iter::<DescriptorSet>().enumerate() {
            descriptor_set_layout_defs.push(into_cgen_descriptor_set(ctx.model, descriptor_set));
        }

        for (_, pipeline_layout) in ctx.model.ref_iter::<PipelineLayout>().enumerate() {
            root_signature_defs.push(into_cgen_pipeline_layout_def(ctx.model, pipeline_layout));
        }
    */
    let cgen_def = CGenDef {
        // type_defs,
        // descriptor_set_layout_defs,
        // root_signature_defs,
    };

    vec![Product::new(
        CGenVariant::Blob,
        RelativePath::new("cgen_def.blob").to_owned(),
        bincode::serialize(&cgen_def).unwrap(),
    )]
}

/*
fn into_cgen_type(_model: &Model, ty_ref: CGenTypeRef) -> CGenTypeDef {
    CGenTypeDef {
        id: CGenTypeId(ty_ref.id()),
    }
}

fn into_cgen_descriptor_set(model: &Model, ds_ref: DescriptorSetRef) -> CGenDescriptorSetDef {
    let ds = ds_ref.get(model);

    let mut flat_index = 0u32;
    let descriptor_defs = ds
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

            let def = CGenDescriptorDef {
                name: d.name.clone(),
                flat_index,
                array_size: d.array_len.unwrap_or(0),
                shader_resource_type,
            };

            flat_index += def.array_size.max(1u32);

            def
        })
        .collect::<Vec<CGenDescriptorDef>>();

    CGenDescriptorSetDef {
        name: ds.name.clone(),
        id: CGenDescriptorSetId(ds_ref.id()),
        frequency: ds.frequency,
        descriptor_flat_count: flat_index,
        descriptor_defs,
    }
}

fn into_cgen_pipeline_layout_def(
    model: &Model,
    pipeline_layout_ref: PipelineLayoutRef,
) -> CGenPipelineLayoutDef {
    let pipeline_layout = pipeline_layout_ref.get(model);

    let mut descriptor_set_layout_ids: [Option<CGenDescriptorSetId>; MAX_DESCRIPTOR_SET_LAYOUTS] =
        [None; MAX_DESCRIPTOR_SET_LAYOUTS];
    pipeline_layout.descriptor_sets().for_each(|ds_ref| {
        let ds = ds_ref.get(model);
        descriptor_set_layout_ids[ds.frequency as usize] = Some(CGenDescriptorSetId(ds_ref.id()));
    });

    CGenPipelineLayoutDef {
        name: pipeline_layout.name.clone(),
        id: CGenPipelineLayoutId(pipeline_layout_ref.id()),
        descriptor_set_layout_ids,
        push_constant_type: None,
    }
}
*/
