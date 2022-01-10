use crate::{
    db::PipelineLayout,
    generators::{file_writer::FileWriter, product::Product, GeneratorContext},
    run::CGenVariant,
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for pipeline_layout_ref in model.object_iter::<PipelineLayout>() {
        let content = generate_hlsl_pipelinelayout(ctx, pipeline_layout_ref.object());
        products.push(Product::new(
            CGenVariant::Hlsl,
            GeneratorContext::get_object_rel_path(pipeline_layout_ref.object(), CGenVariant::Hlsl),
            content.into_bytes(),
        ));
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
    let mut pl_folder = GeneratorContext::get_object_rel_path(pl, CGenVariant::Hlsl);
    pl_folder.pop();
    writer.add_line("// DescriptorSets");
    for (name, ty) in &pl.members {
        match ty {
            crate::db::PipelineLayoutContent::DescriptorSet(ds_handle) => {
                let ds = ds_handle.get(ctx.model);
                let ds_path = GeneratorContext::get_object_rel_path(ds, CGenVariant::Hlsl);
                let rel_path = pl_folder.relative(ds_path);
                writer.add_line(format!("// - name: {}", name));
                writer.add_line(format!("// - freq: {}", ds.frequency));
                writer.add_line(format!("#include \"{}\"", rel_path));
                writer.new_line();
            }
            crate::db::PipelineLayoutContent::Pushconstant(_) => (),
        }
    }
    writer.add_line("// PushConstant".to_string());
    for (name, ty) in &pl.members {
        match ty {
            crate::db::PipelineLayoutContent::Pushconstant(ty_ref) => {
                let ty = ty_ref.get(ctx.model);
                let ty_path = GeneratorContext::get_object_rel_path(ty, CGenVariant::Hlsl);
                let rel_path = pl_folder.relative(ty_path);
                writer.add_line(format!("// - name: {}", name));
                writer.add_line(format!("#include \"{}\"", rel_path));
                writer.new_line();
                writer.add_line("[[vk::push_constant]]");
                writer.add_line(format!("ConstantBuffer<{}> {}; ", ty.name(), name));
                writer.new_line();
            }
            crate::db::PipelineLayoutContent::DescriptorSet(_) => (),
        }
    }

    writer.new_line();
    writer.unindent();

    // footer
    writer.add_line("#endif");

    // finalize
    writer.build()
}
