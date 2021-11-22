use heck::SnakeCase;
use relative_path::RelativePath;

use crate::{generators::{
        file_writer::FileWriter, product::Product, rust::utils::get_rust_typestring, CGenVariant,
        GeneratorContext,
    }, model::{CGenType, Descriptor, DescriptorSet, Model, StructMember}};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    let descriptor_sets = model.object_iter::<DescriptorSet>().unwrap_or_default();
    for descriptor_set in descriptor_sets {        
        let content = generate_rust_descriptorset(&ctx, descriptor_set);
        products.push(Product::new(
                CGenVariant::Rust,
                GeneratorContext::get_object_rel_path(descriptor_set, CGenVariant::Rust),                
                content,
            ));        
    }

    if !products.is_empty() {
        let mut mod_path = GeneratorContext::get_object_folder::<DescriptorSet>();
        mod_path.push("mod.rs");        

        let mut writer = FileWriter::new();
        for product in &products {
            let filename = product.path().file_stem().unwrap();
            writer.add_line(format!("pub(crate) mod {};", &filename));
        }
        products.push(Product::new(
            CGenVariant::Rust,
            mod_path,
            writer.to_string(),
        ));
    }

    products
}

fn generate_rust_descriptorset(ctx: &GeneratorContext<'_>, descriptor_set: &DescriptorSet) -> String{
    String::new()
}