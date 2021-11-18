use relative_path::RelativePath;

use crate::{
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
    model::{CGenType, DescriptorSet, PipelineLayout},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let mut writer = FileWriter::new();

    let model = ctx.model;
    if model.size::<CGenType>() > 0 {
        writer.add_line("pub(crate) mod types;".to_string());
    }
    if model.size::<DescriptorSet>() > 0 {
        writer.add_line("pub(crate) mod descriptorsets;".to_string());
    }
    if model.size::<PipelineLayout>() > 0 {
        writer.add_line("pub(crate) mod pipelinelayouts;".to_string());
    }

    products.push(Product::new(
        CGenVariant::Rust,
        RelativePath::new("mod.rs").to_relative_path_buf(),
        writer.to_string(),
    ));

    products
}
