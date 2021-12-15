use heck::SnakeCase;

use crate::{
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
    model::{CGenType, DescriptorSet},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for descriptor_set in model.object_iter::<DescriptorSet>() {
        let content = generate_rust_descriptorset(ctx, descriptor_set);
        products.push(Product::new(
            CGenVariant::Rust,
            GeneratorContext::get_object_rel_path(descriptor_set, CGenVariant::Rust),
            content.into_bytes(),
        ));
    }

    if !products.is_empty() {
        let mut mod_path = GeneratorContext::get_object_folder::<DescriptorSet>();
        mod_path.push("mod.rs");

        let mut writer = FileWriter::new();
        for product in &products {
            let filename = product.path().file_stem().unwrap();
            writer.add_line(format!("pub(crate) mod {};", &filename));
            writer.add_line("#[allow(unused_imports)]");
            writer.add_line(format!("pub(crate) use {}::*;", &filename));
        }
        products.push(Product::new(
            CGenVariant::Rust,
            mod_path,
            writer.to_string().into_bytes(),
        ));
    }

    products
}

fn generate_rust_descriptorset(
    ctx: &GeneratorContext<'_>,
    descriptor_set: &DescriptorSet,
) -> String {
    let mut writer = FileWriter::new();

    // global dependencies
    writer.add_line("use lgn_graphics_api::DeviceContext;");
    writer.add_line("use lgn_graphics_api::DescriptorSetLayoutDef;");
    writer.add_line("use lgn_graphics_api::DescriptorSetLayout;");

    // local dependencies
    let deps = GeneratorContext::get_descriptorset_dependencies(descriptor_set);

    if !deps.is_empty() {
        for object_id in &deps {
            let dep_ty = ctx.model.get_from_objectid::<CGenType>(*object_id).unwrap();
            match dep_ty {
                CGenType::Native(_) => {}
                CGenType::Struct(e) => {
                    writer.add_line("#[allow(unused_imports)]");
                    writer.add_line(format!(
                        "use super::super::c_gen_type::{}::{};",
                        e.name.to_snake_case(),
                        e.name
                    ));
                }
            }
        }
        writer.new_line();
    }

    // struct
    writer.add_line(format!("pub struct {} {{", descriptor_set.name));
    writer.indent();
    writer.add_line("api_layout : DescriptorSetLayout,");
    writer.unindent();
    writer.add_line("}".to_string());
    writer.new_line();
    // trait
    writer.add_line(format!("impl {} {{", descriptor_set.name));
    writer.indent();
    // new
    writer.add_line("pub fn new(device_context: &DeviceContext) -> Self {".to_string());
    writer.indent();
    writer.add_line("let mut layout_def = DescriptorSetLayoutDef::default();");
    writer.add_line(format!(
        "layout_def.frequency = {};",
        descriptor_set.frequency
    ));
    for _descriptor_def in &descriptor_set.descriptors {}
    writer.add_line(
        "let api_layout = device_context.create_descriptorset_layout(&layout_def).unwrap();",
    );
    writer.add_line("Self { api_layout }".to_string());
    writer.unindent();
    writer.add_line("}".to_string());
    // api_layout
    writer.add_line(
        "pub fn api_layout(&self) -> &DescriptorSetLayout { &self.api_layout }".to_string(),
    );
    writer.unindent();
    writer.add_line("}".to_string());

    // finalize
    writer.to_string()
}
