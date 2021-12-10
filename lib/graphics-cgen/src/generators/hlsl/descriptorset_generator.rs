use crate::{
    generators::{
        file_writer::FileWriter, hlsl::utils::get_hlsl_typestring, product::Product,
        GeneratorContext,
    },
    model::{CGenType, Descriptor, DescriptorDef, DescriptorSet, Model},
    run::CGenVariant,
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for descriptorset in model.object_iter::<DescriptorSet>() {
        let content = generate_hlsl_descritporset(ctx, descriptorset);
        products.push(Product::new(
            CGenVariant::Hlsl,
            GeneratorContext::get_object_rel_path(descriptorset, CGenVariant::Hlsl),
            content.into_bytes(),
        ))
    }
    products
}

fn get_descriptor_declaration(model: &Model, descriptor: &Descriptor) -> String {
    let type_name: String = match &descriptor.def {
        DescriptorDef::Sampler => "SamplerState ".to_owned(),
        DescriptorDef::ConstantBuffer(def) => {
            format!(
                "ConstantBuffer<{}>",
                get_hlsl_typestring(model, def.object_id)
            )
        }
        DescriptorDef::StructuredBuffer(def) => {
            format!(
                "StructuredBuffer<{}>",
                get_hlsl_typestring(model, def.object_id)
            )
        }
        DescriptorDef::RWStructuredBuffer(def) => {
            format!(
                "RWStructuredBuffer<{}>",
                get_hlsl_typestring(model, def.object_id)
            )
        }
        DescriptorDef::ByteAddressBuffer => "ByteAddressBuffer".to_owned(),
        DescriptorDef::RWByteAddressBuffer => "RWByteAddressBuffer".to_owned(),
        DescriptorDef::Texture2D(def) => {
            format!("Texture2D<{}>", get_hlsl_typestring(model, def.object_id))
        }
        DescriptorDef::RWTexture2D(def) => {
            format!("RWTexture2D<{}>", get_hlsl_typestring(model, def.object_id))
        }
        DescriptorDef::Texture3D(def) => {
            format!("Texture3D<{}>", get_hlsl_typestring(model, def.object_id))
        }
        DescriptorDef::RWTexture3D(def) => {
            format!("RWTexture3D<{}>", get_hlsl_typestring(model, def.object_id))
        }
        DescriptorDef::Texture2DArray(def) => {
            format!(
                "Texture2DArray<{}>",
                get_hlsl_typestring(model, def.object_id)
            )
        }
        DescriptorDef::RWTexture2DArray(def) => {
            format!(
                "RWTexture2DArray<{}>",
                get_hlsl_typestring(model, def.object_id)
            )
        }
        DescriptorDef::TextureCube(def) => {
            format!("TextureCube<{}>", get_hlsl_typestring(model, def.object_id))
        }
        DescriptorDef::TextureCubeArray(def) => {
            format!(
                "TextureCubeArray<{}>",
                get_hlsl_typestring(model, def.object_id)
            )
        }
    };

    if let Some(array_len) = descriptor.array_len {
        format!("{} {}[{}];", type_name, descriptor.name, array_len)
    } else {
        format!("{} {};", type_name, descriptor.name)
    }
}

fn generate_hlsl_descritporset(ctx: &GeneratorContext<'_>, ds: &DescriptorSet) -> String {
    let mut writer = FileWriter::new();

    // header
    writer.add_line(format!("#ifndef DESCRIPTORSET_{}", ds.name.to_uppercase()));
    writer.add_line(format!("#define DESCRIPTORSET_{}", ds.name.to_uppercase()));
    writer.new_line();

    writer.indent();

    // include all type dependencies
    let deps = GeneratorContext::get_descriptorset_dependencies(ds);

    if !deps.is_empty() {
        let mut cur_folder = GeneratorContext::get_object_rel_path(ds, CGenVariant::Hlsl);
        cur_folder.pop();
        for object_id in deps.iter() {
            let ty = ctx.model.get_from_objectid::<CGenType>(*object_id).unwrap();
            let ty_path = GeneratorContext::get_object_rel_path(ty, CGenVariant::Hlsl);
            let rel_path = cur_folder.relative(ty_path);
            writer.add_line(format!("#include \"{}\"", rel_path));
        }
        writer.new_line();
    }

    for (idx, d) in ds.descriptors.iter().enumerate() {
        writer.add_line(format!("[[vk::binding({}, {})]]", idx, ds.frequency));
        writer.add_line(get_descriptor_declaration(ctx.model, d));
    }

    writer.new_line();
    writer.unindent();

    // footer
    writer.add_line("#endif".to_string());

    // finalize
    writer.to_string()
}
