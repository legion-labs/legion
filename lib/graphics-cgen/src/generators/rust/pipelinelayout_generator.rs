
use crate::{generators::{
        file_writer::FileWriter, product::Product, CGenVariant,
        GeneratorContext,
    }, model::{PipelineLayout}};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    let pipeline_layouts = model.object_iter::<PipelineLayout>().unwrap_or_default();
    for pipeline_layout in pipeline_layouts {        
        let content = generate_rust_pipeline_layout(&ctx, pipeline_layout);
        products.push(Product::new(
                CGenVariant::Rust,
                GeneratorContext::get_object_rel_path(pipeline_layout, CGenVariant::Rust),                
                content,
            ));        
    }

    if !products.is_empty() {
        let mut mod_path = GeneratorContext::get_object_folder::<PipelineLayout>();
        mod_path.push("mod.rs");        

        let mut writer = FileWriter::new();
        for product in &products {
            let filename = product.path().file_stem().unwrap();
            writer.add_line(format!("pub(crate) mod {};", &filename));
            writer.add_line(format!("pub(crate) use {}::*;", &filename));
        }
        products.push(Product::new(
            CGenVariant::Rust,
            mod_path,
            writer.to_string(),
        ));
    }

    products
}

fn generate_rust_pipeline_layout(_ctx: &GeneratorContext<'_>, _pipeline_layout: &PipelineLayout) -> String{
    String::new()
}