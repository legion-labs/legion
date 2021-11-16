use crate::{
    generators::{
        hlsl::utils::get_hlsl_typestring, file_writer::FileWriter, CGenVariant, Generator,
        GeneratorContext, Product,
    },
    model::{Descriptor, DescriptorDef, DescriptorSet, Model, PipelineLayout},
};

#[derive(Default)]
pub struct PipelineLayoutGenerator;

impl Generator for PipelineLayoutGenerator {
    fn run(&self, ctx: &GeneratorContext<'_>) -> Vec<Product> {
        let mut products = Vec::new();
        let model = ctx.model;
        let pipeline_layouts = model.object_iter::<PipelineLayout>().unwrap_or_default();    
        for pipeline_layout in pipeline_layouts {
            let content = generate_hlsl_pipelinelayout(ctx, pipeline_layout);
            products.push(Product {
                path: ctx.get_pipelinelayout_abspath(pipeline_layout, CGenVariant::Hlsl),
                content,
            });
        }        
        products
    }
}

fn get_descriptor_declaration(model: &Model, descriptor: &Descriptor) -> String {
    let typestring: String = match &descriptor.def {
        DescriptorDef::Sampler => "SamplerState ".to_owned(),
        DescriptorDef::ConstantBuffer(cb_def) => {
            format!(
                "ConstantBuffer<{}>",
                get_hlsl_typestring(model, cb_def.type_key)
            )
        }
        DescriptorDef::StructuredBuffer(sb_def) => {
            format!(
                "StructuredBuffer<{}>",
                get_hlsl_typestring(model, sb_def.type_key)
            )
        }
        DescriptorDef::RWStructuredBuffer(sb_def) => {
            format!(
                "RWStructuredBuffer<{}>",
                get_hlsl_typestring(model, sb_def.type_key)
            )
        }
        DescriptorDef::ByteAddressBuffer => "ByteAddressBuffer".to_owned(),
        DescriptorDef::RWByteAddressBuffer => "RWByteAddressBuffer".to_owned(),
        DescriptorDef::Texture2D(t_def) => {
            format!("Texture2D<{}>", get_hlsl_typestring(model, t_def.type_key))
        }
        DescriptorDef::RWTexture2D(t_def) => {
            format!(
                "RWTexture2D<{}>",
                get_hlsl_typestring(model, t_def.type_key)
            )
        }
    };

    format!("{} {};", typestring, descriptor.name)
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
    if !pl.descriptorsets.is_empty() {
        for ds_id in pl.descriptorsets.iter() {
            let ds = ctx.model.get::<DescriptorSet>(*ds_id).unwrap();
            writer.add_line(format!(
                "// DescriptorSet '{}' : freq '{}'",
                ds.name, ds.frequency
            ));

            for (idx, d) in ds.descriptors.iter().enumerate() {
                writer.add_line(format!("[[vk::binding({}, {})]]", idx, ds.frequency));
                writer.add_line(get_descriptor_declaration(ctx.model, d));
            }
        }
        writer.new_line();
    }

    writer.unindent();

    // footer
    writer.add_line("#endif".to_string());

    // finalize
    writer.to_string()
}
