use crate::{
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
    model::{DescriptorSet, DescriptorSetRef},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for descriptor_set_ref in model.ref_iter::<DescriptorSet>() {
        let descriptor_set = descriptor_set_ref.get(model);
        let content = generate_rust_descriptorset(ctx, descriptor_set_ref);
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

fn generate_rust_descriptorset(
    ctx: &GeneratorContext<'_>,
    descriptor_set_ref: DescriptorSetRef,
) -> String {
    let descriptor_set = descriptor_set_ref.get(ctx.model);

    let mut writer = FileWriter::new();

    // global dependencies
    // writer.add_line("use lgn_graphics_api::DeviceContext;");
    // writer.add_line("use lgn_graphics_api::DescriptorSetLayoutDef;");
    // writer.add_line("use lgn_graphics_api::DescriptorSetLayout;");
    writer.add_line("use lgn_graphics_cgen_runtime::CGenDescriptorSetId;");
    writer.add_line("use lgn_graphics_cgen_runtime::CGenDescriptorSetInfo;");
    writer.add_line("use lgn_graphics_cgen_runtime::CGenDescriptorId;");
    writer.new_line();
    // local dependencies
    /*
    let deps = GeneratorContext::get_descriptorset_dependencies(descriptor_set);

    if !deps.is_empty() {
        for ty_ref in &deps {
            let ty = ty_ref.get(ctx.model);
            match ty {
                CGenType::Native(_) => {}
                CGenType::Struct(e) => {
                    writer.add_line("#[allow(unused_imports)]");
                    writer.add_line(format!(
                        "use super::super::cgen_type::{}::{};",
                        e.name.to_snake_case(),
                        e.name
                    ));
                }
            }
        }
        writer.new_line();
    }
    */

    // id
    writer.add_line(format!(
        "static ID : CGenDescriptorSetId = CGenDescriptorSetId({}); ",
        descriptor_set_ref.id()
    ));
    writer.new_line();
    // struct
    writer.add_line(format!("pub struct {};", descriptor_set.name));
    writer.new_line();
    // impl
    writer.add_line("#[allow(non_upper_case_globals)]");
    writer.add_line(format!("impl {} {{", descriptor_set.name));
    writer.indent();
    for (idx, descriptor) in descriptor_set.descriptors.iter().enumerate() {
        writer.add_line(format!(
            "pub const {} : CGenDescriptorId = CGenDescriptorId({});",
            descriptor.name, idx
        ));
    }
    writer.new_line();
    writer.add_line("pub fn id() -> CGenDescriptorSetId { ID }");
    writer.unindent();
    writer.add_line(format!("}} // {}", descriptor_set.name));
    writer.new_line();
    // trait info
    writer.add_line(format!(
        "impl CGenDescriptorSetInfo for {} {{",
        descriptor_set.name
    ));
    writer.indent();
    // writer.add_line(format!(
    //     "type DescriptorID = {}DescriptorID;",
    //     descriptor_set.name
    // ));
    writer.add_line("fn id() -> CGenDescriptorSetId { Self::id() }");
    writer.unindent();
    writer.add_line("} // CGenDescriptorSetInfo");
    writer.new_line();

    /*
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
    */
    // finalize
    writer.build()
}
