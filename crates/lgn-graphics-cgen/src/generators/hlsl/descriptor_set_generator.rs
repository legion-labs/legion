use crate::{
    db::{Descriptor, DescriptorDef, DescriptorSet, Model},
    generators::{file_writer::FileWriter, product::Product, GeneratorContext},
    run::CGenVariant,
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for descriptor_set_ref in model.object_iter::<DescriptorSet>() {
        let content = generate_hlsl_descriptor_set(ctx, descriptor_set_ref.object());
        products.push(Product::new(
            CGenVariant::Hlsl,
            GeneratorContext::object_relative_path(descriptor_set_ref.object(), CGenVariant::Hlsl),
            content.into_bytes(),
        ));
    }
    products
}

fn descriptor_declaration(model: &Model, descriptor: &Descriptor) -> String {
    let type_name: String = match &descriptor.def {
        DescriptorDef::Sampler => "SamplerState ".to_owned(),
        DescriptorDef::ConstantBuffer(def) => {
            format!(
                "ConstantBuffer<{}>",
                def.ty_handle.get(model).to_hlsl_name()
            )
        }
        DescriptorDef::StructuredBuffer(def) => {
            format!(
                "StructuredBuffer<{}>",
                def.ty_handle.get(model).to_hlsl_name()
            )
        }
        DescriptorDef::RWStructuredBuffer(def) => {
            format!(
                "RWStructuredBuffer<{}>",
                def.ty_handle.get(model).to_hlsl_name()
            )
        }
        DescriptorDef::ByteAddressBuffer => "ByteAddressBuffer".to_owned(),
        DescriptorDef::RWByteAddressBuffer => "RWByteAddressBuffer".to_owned(),
        DescriptorDef::Texture2D(def) => {
            format!("Texture2D<{}>", def.ty_handle.get(model).to_hlsl_name())
        }
        DescriptorDef::RWTexture2D(def) => {
            format!("RWTexture2D<{}>", def.ty_handle.get(model).to_hlsl_name())
        }
        DescriptorDef::Texture3D(def) => {
            format!("Texture3D<{}>", def.ty_handle.get(model).to_hlsl_name())
        }
        DescriptorDef::RWTexture3D(def) => {
            format!("RWTexture3D<{}>", def.ty_handle.get(model).to_hlsl_name())
        }
        DescriptorDef::Texture2DArray(def) => {
            format!(
                "Texture2DArray<{}>",
                def.ty_handle.get(model).to_hlsl_name()
            )
        }
        DescriptorDef::RWTexture2DArray(def) => {
            format!(
                "RWTexture2DArray<{}>",
                def.ty_handle.get(model).to_hlsl_name()
            )
        }
        DescriptorDef::TextureCube(def) => {
            format!("TextureCube<{}>", def.ty_handle.get(model).to_hlsl_name())
        }
        DescriptorDef::TextureCubeArray(def) => {
            format!(
                "TextureCubeArray<{}>",
                def.ty_handle.get(model).to_hlsl_name()
            )
        }
    };

    if let Some(array_len) = descriptor.array_len {
        format!("{} {}[{}];", type_name, descriptor.name, array_len)
    } else {
        format!("{} {};", type_name, descriptor.name)
    }
}

fn generate_hlsl_descriptor_set(ctx: &GeneratorContext<'_>, ds: &DescriptorSet) -> String {
    let mut writer = FileWriter::new();

    // header
    {
        let mut writer = writer.add_block(
            &[
                format!("#ifndef DESCRIPTOR_SET_{}", ds.name.to_uppercase()),
                format!("#define DESCRIPTOR_SET_{}", ds.name.to_uppercase()),
            ],
            &["#endif"],
        );
        writer.new_line();
        let deps = ds.get_type_dependencies();
        if !deps.is_empty() {
            let mut includes = deps
                .iter()
                .filter_map(|ty_ref| {
                    let ty = ty_ref.get(ctx.model);
                    match ty {
                        crate::db::CGenType::Native(_) => None,
                        crate::db::CGenType::Struct(_) => Some(format!(
                            "#include \"{}\"",
                            ctx.embedded_fs_path(ty, CGenVariant::Hlsl)
                        )),
                        crate::db::CGenType::BitField(_) => Some(format!(
                            "#include \"{}\"",
                            ctx.embedded_fs_path(ty, CGenVariant::Hlsl)
                        )),
                    }
                })
                .collect::<Vec<_>>();
            includes.sort();
            includes.into_iter().for_each(|i| writer.add_line(i));

            writer.new_line();
        }

        for (idx, d) in ds.descriptors.iter().enumerate() {
            writer.add_lines(&[
                format!("[[vk::binding({}, {})]]", idx, ds.frequency),
                descriptor_declaration(ctx.model, d),
            ]);
        }
    }

    // include all type dependencies

    // finalize
    writer.build()
}
