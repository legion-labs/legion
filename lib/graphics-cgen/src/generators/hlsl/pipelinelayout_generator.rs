use crate::{
    generators::{
        file_writer::FileWriter, hlsl::utils::get_hlsl_typestring, product::Product,
        GeneratorContext,
    },
    model::{Descriptor, DescriptorDef, DescriptorSet, Model, PipelineLayout},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    let pipeline_layouts = model.object_iter::<PipelineLayout>().unwrap_or_default();
    for pipeline_layout in pipeline_layouts {
        let content = generate_hlsl_pipelinelayout(ctx, pipeline_layout);
        todo!();
        // products.push(Product::new(
        //     ctx.get_pipelinelayout_abspath(pipeline_layout, CGenVariant::Hlsl),
        //     content,
        // ));
    }
    products
}

fn get_descriptor_declaration(model: &Model, descriptor: &Descriptor) -> String {
    let type_name: String = match &descriptor.def {
        DescriptorDef::Sampler => "SamplerState ".to_owned(),
        DescriptorDef::ConstantBuffer(def) => {
            format!(
                "ConstantBuffer<{}>",
                get_hlsl_typestring(model, def.type_key)
            )
        }
        DescriptorDef::StructuredBuffer(def) => {
            format!(
                "StructuredBuffer<{}>",
                get_hlsl_typestring(model, def.type_key)
            )
        }
        DescriptorDef::RWStructuredBuffer(def) => {
            format!(
                "RWStructuredBuffer<{}>",
                get_hlsl_typestring(model, def.type_key)
            )
        }
        DescriptorDef::ByteAddressBuffer => "ByteAddressBuffer".to_owned(),
        DescriptorDef::RWByteAddressBuffer => "RWByteAddressBuffer".to_owned(),
        DescriptorDef::Texture2D(def) => {
            format!("Texture2D<{}>", get_hlsl_typestring(model, def.type_key))
        }
        DescriptorDef::RWTexture2D(def) => {
            format!("RWTexture2D<{}>", get_hlsl_typestring(model, def.type_key))
        }
        DescriptorDef::Texture3D(def) => {
            format!("Texture3D<{}>", get_hlsl_typestring(model, def.type_key))
        }
        DescriptorDef::RWTexture3D(def) => {
            format!("RWTexture3D<{}>", get_hlsl_typestring(model, def.type_key))
        }
        DescriptorDef::Texture2DArray(def) => {
            format!(
                "Texture2DArray<{}>",
                get_hlsl_typestring(model, def.type_key)
            )
        }
        DescriptorDef::RWTexture2DArray(def) => {
            format!(
                "RWTexture2DArray<{}>",
                get_hlsl_typestring(model, def.type_key)
            )
        }
        DescriptorDef::TextureCube(def) => {
            format!("TextureCube<{}>", get_hlsl_typestring(model, def.type_key))
        }
        DescriptorDef::TextureCubeArray(def) => {
            format!(
                "TextureCubeArray<{}>",
                get_hlsl_typestring(model, def.type_key)
            )
        }
    };

    if let Some(array_len) = descriptor.array_len {
        format!("{} {}[{}];", type_name, descriptor.name, array_len)
    } else {
        format!("{} {};", type_name, descriptor.name)
    }
}

fn generate_hlsl_pipelinelayout(ctx: &GeneratorContext<'_>, pl: &PipelineLayout) -> String {
    let mut writer = FileWriter::new();

    // header
    writer.add_line(format!("#ifndef PIPELINELAYOUT_{}", pl.name.to_uppercase()));
    writer.add_line(format!("#define PIPELINELAYOUT_{}", pl.name.to_uppercase()));
    writer.new_line();

    writer.indent();

    // include all type dependencies
    // let deps = context
    //    .model
    //     .get_pipelinelayout_type_dependencies(pl_name)
    //     .unwrap();

    // if !deps.is_empty() {
    //     for dep in deps.iter() {
    //         writer.add_line(format!("#include \"../structs/{}.hlsl\"", dep.to_string()));
    //     }
    //     writer.new_line();
    // }

    // write all descriptorsets
    // if !pl.descriptorsets.is_empty() {
    //     for ds_id in pl.descriptorsets.iter() {
    //         let ds = ctx.model.get::<DescriptorSet>(*ds_id).unwrap();
    //         writer.add_line(format!(
    //             "// DescriptorSet '{}' : freq '{}'",
    //             ds.name, ds.frequency
    //         ));

    //         for (idx, d) in ds.descriptors.iter().enumerate() {
    //             writer.add_line(format!("[[vk::binding({}, {})]]", idx, ds.frequency));
    //             writer.add_line(get_descriptor_declaration(ctx.model, d));
    //         }
    //     }
    //     writer.new_line();
    // }

    writer.unindent();

    // footer
    writer.add_line("#endif".to_string());

    // finalize
    writer.to_string()
}
