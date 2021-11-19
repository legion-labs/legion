use crate::{
    generators::{
        file_writer::FileWriter, hlsl::utils::get_hlsl_typestring, product::Product,
        GeneratorContext,
    },
    model::{Descriptor, DescriptorDef, DescriptorSet, Model, PipelineLayout},
    run::CGenVariant,
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    let pipeline_layouts = model.object_iter::<PipelineLayout>().unwrap_or_default();
    for pipeline_layout in pipeline_layouts {
        let content = generate_hlsl_pipelinelayout(ctx, pipeline_layout);
        products.push(Product::new(
            CGenVariant::Hlsl,
            GeneratorContext::get_object_rel_path(pipeline_layout, CGenVariant::Hlsl),
            content,
        ))
    }
    products
}

fn generate_hlsl_pipelinelayout(ctx: &GeneratorContext<'_>, pl: &PipelineLayout) -> String {
    let mut writer = FileWriter::new();

    // header
    writer.add_line(format!("#ifndef PIPELINELAYOUT_{}", pl.name.to_uppercase()));
    writer.add_line(format!("#define PIPELINELAYOUT_{}", pl.name.to_uppercase()));
    writer.new_line();

    writer.indent();

    // include all type dependencies
    writer.add_line(format!("// DescriptorSets"));
    let mut pl_folder = GeneratorContext::get_object_rel_path(pl, CGenVariant::Hlsl);
    pl_folder.pop();
    for (name, ty) in &pl.members {
        match ty {
            crate::model::PipelineLayoutContent::DescriptorSet(def) => {
                let ds = ctx.model.get::<DescriptorSet>(*def).unwrap();
                let ds_path = GeneratorContext::get_object_rel_path(ds, CGenVariant::Hlsl);
                let rel_path = pl_folder.relative(ds_path);
                writer.add_line(format!("// - name: {}", name));
                writer.add_line(format!("// - freq: {}", ds.frequency));
                writer.add_line(format!("#include \"{}\"", rel_path));
                writer.new_line();
            }
            crate::model::PipelineLayoutContent::Pushconstant(_) => (),
        }
    }

    writer.new_line();
    writer.unindent();

    // footer
    writer.add_line("#endif".to_string());

    // finalize
    writer.to_string()
}
