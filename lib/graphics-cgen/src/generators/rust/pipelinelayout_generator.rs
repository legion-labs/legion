use crate::{
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
    model::{PipelineLayout, PipelineLayoutRef},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for pipeline_layout_ref in model.ref_iter::<PipelineLayout>() {
        let pipeline_layout = pipeline_layout_ref.get(model);
        let content = generate_rust_pipeline_layout(ctx, pipeline_layout_ref);
        products.push(Product::new(
            CGenVariant::Rust,
            GeneratorContext::get_object_rel_path(pipeline_layout, CGenVariant::Rust),
            content.into_bytes(),
        ));
    }

    if !products.is_empty() {
        let mut mod_path = GeneratorContext::get_object_folder::<PipelineLayout>();
        mod_path.push("mod.rs");

        let mut writer = FileWriter::new();
        for product in &products {
            let filename = product.path().file_stem().unwrap();
            writer.add_line(format!("pub mod {};", &filename));
            writer.add_line("#[allow(unused_imports)]");
            writer.add_line(format!("pub use {}::*;", &filename));
        }
        products.push(Product::new(
            CGenVariant::Rust,
            mod_path,
            writer.build().into_bytes(),
        ));
    }

    products
}

fn generate_rust_pipeline_layout(
    ctx: &GeneratorContext<'_>,
    pipeline_layout_ref: PipelineLayoutRef,
) -> String {
    let pipeline_layout = pipeline_layout_ref.get(ctx.model);

    let mut writer = FileWriter::new();

    // global dependencies
    /*
    writer.add_line("#[allow(unused_imports)]");
    writer.add_line("use lgn_graphics_api::DeviceContext;");
    writer.add_line("#[allow(unused_imports)]");
    writer.add_line("use lgn_graphics_api::DescriptorSetLayoutDef;");
    writer.add_line("#[allow(unused_imports)]");
    writer.add_line("use lgn_graphics_api::DescriptorSetLayout;");
    */
    writer.add_line("use lgn_graphics_cgen_runtime::CGenPipelineLayoutId;");
    writer.add_line("use lgn_graphics_cgen_runtime::CGenPipelineLayoutInfo;");
    writer.new_line();

    // id
    writer.add_line(format!(
        "static ID : CGenPipelineLayoutId = CGenPipelineLayoutId({}); ",
        pipeline_layout_ref.id()
    ));
    writer.new_line();
    // struct
    writer.add_line(format!("pub struct {};", pipeline_layout.name));
    writer.new_line();
     // impl
     writer.add_line(format!("impl {} {{", pipeline_layout.name));
     writer.indent();
     writer.add_line("pub fn id() -> CGenPipelineLayoutId { ID }");
     writer.unindent();
     writer.add_line(format!("}} // {}", pipeline_layout.name));
     writer.new_line();
    // trait info
    writer.add_line(format!(
        "impl CGenPipelineLayoutInfo for {} {{",
        pipeline_layout.name
    ));
    writer.indent();
    writer.add_line("fn id() -> CGenPipelineLayoutId { ID }");
    writer.unindent();
    writer.add_line("} // CGenPipelineLayoutInfo");
    writer.new_line();

    // finalize
    writer.build()
}
