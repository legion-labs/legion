use crate::{
    generators::{
        file_writer::FileWriter, product::Product,
        GeneratorContext,
    },
    model::{
        CGenType, DescriptorSet, ModelObject, PipelineLayout,
    },
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
    let mut pl_folder = GeneratorContext::get_object_rel_path(pl, CGenVariant::Hlsl);
    pl_folder.pop();
    writer.add_line(format!("// DescriptorSets"));
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
    writer.add_line(format!("// PushConstant"));
    for (name, ty) in &pl.members {
        match ty {
            crate::model::PipelineLayoutContent::Pushconstant(def) => {
                let ty = ctx.model.get::<CGenType>(*def).unwrap();
                let ty_path = GeneratorContext::get_object_rel_path(ty, CGenVariant::Hlsl);
                let rel_path = pl_folder.relative(ty_path);
                writer.add_line(format!("// - name: {}", name));
                writer.add_line(format!("#include \"{}\"", rel_path));
                writer.new_line();
                writer.add_line(format!("[[vk::push_constant]]"));
                writer.add_line(format!("ConstantBuffer<{}> {}; ", ty.name(), name));
                writer.new_line();
            }
            crate::model::PipelineLayoutContent::DescriptorSet(_) => (),
        }
    }

    writer.new_line();
    writer.unindent();

    // footer
    writer.add_line("#endif".to_string());

    // finalize
    writer.to_string()
}
