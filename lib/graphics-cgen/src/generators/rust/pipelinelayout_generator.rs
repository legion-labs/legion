use lgn_graphics_api::MAX_DESCRIPTOR_SET_LAYOUTS;

use crate::{
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
    model::{ModelObject, PipelineLayout, PipelineLayoutRef},
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
    writer.add_line("use lgn_graphics_api::DeviceContext;");
    writer.add_line("use lgn_graphics_api::RootSignature;");
    writer.add_line("use lgn_graphics_api::DescriptorSetLayout;");
    writer.add_line("use lgn_graphics_api::DescriptorSetHandle;");
    writer.add_line("use lgn_graphics_api::MAX_DESCRIPTOR_SET_LAYOUTS;");
    writer.add_line("use lgn_graphics_cgen_runtime::CGenPipelineLayoutDef;");
    writer.new_line();

    // local dependencies
    for (name, content) in &pipeline_layout.members {
        match content {
            crate::model::PipelineLayoutContent::DescriptorSet(ds_ref) => {
                let ds = ds_ref.get(ctx.model);
                writer.add_line(format!("use super::super::descriptor_set::{};", ds.name));
            }
            crate::model::PipelineLayoutContent::Pushconstant(typ_ref) => {
                todo!();
            }
        }

        // writer.add_line(format!(
        //     "pub fn set_{}(&mut self, descriptor_set_handle: DescriptorSetHandle) {{",
        //     name
        // ));
        // writer.add_line("}");
    }
    writer.new_line();

    // write cgen pipeline layout def
    writer.add_line("static pipeline_layout_def: CGenPipelineLayoutDef = CGenPipelineLayoutDef{ ");
    writer.indent();
    writer.add_line(format!("name: \"{}\",", pipeline_layout.name));
    writer.add_line(format!("id: {},", pipeline_layout_ref.id()));
    writer.add_line("descriptor_set_layout_ids: [");
    for i in 0..MAX_DESCRIPTOR_SET_LAYOUTS {
        let opt_ds_ref = pipeline_layout.find_descriptor_set_by_frequency(ctx.model, i);
        match opt_ds_ref {
            Some(ds_ref) => {
                let ds = ds_ref.get(ctx.model);
                // writer.add_line(format!("Some({}),", ds_ref.id()))
                writer.add_line(format!("Some({}::id()),", ds.name))
            }
            None => writer.add_line("None,"),
        }
    }
    writer.add_line("],");
    writer.add_line("push_constant_type: None,");

    writer.unindent();
    writer.add_line("}; ");
    writer.new_line();

    // pipeline layout
    writer.add_line("static mut pipeline_layout: Option<RootSignature> = None;");
    writer.new_line();

    // struct
    writer.add_line(format!("pub struct {} {{", pipeline_layout.name));
    writer.indent();
    writer.add_line("descriptor_sets: [Option<DescriptorSetHandle>; MAX_DESCRIPTOR_SET_LAYOUTS],");
    writer.unindent();
    writer.add_line("}");
    writer.new_line();

    // impl
    writer.add_line(format!("impl {} {{", pipeline_layout.name));
    writer.indent();
    writer.new_line();
    writer.add_line("#![allow(unsafe_code)]");
    writer.add_line("pub fn initialize(device_context: &DeviceContext, descriptor_set_layouts: &[&DescriptorSetLayout]) {");
    writer.indent();
    writer.add_line( "unsafe { pipeline_layout = Some(pipeline_layout_def.create_pipeline_layout(device_context, descriptor_set_layouts)); }" );
    writer.unindent();
    writer.add_line("}");
    writer.new_line();

    for (name, content) in &pipeline_layout.members {
        match content {
            crate::model::PipelineLayoutContent::DescriptorSet(ds_ref) => {
                writer.add_line(format!(
                    "pub fn set_{}(&mut self, descriptor_set_handle: DescriptorSetHandle) {{",
                    name
                ));
                writer.indent();
                // writer.add_line("self.descriptor_sets[{}] = ");
                writer.unindent();
                writer.add_line("}");
            }
            crate::model::PipelineLayoutContent::Pushconstant(ty_ref) => {
                todo!();
            }
        }
    }

    writer.unindent();
    writer.add_line("}");
    writer.new_line();

    // id
    // writer.add_line(format!("static ID : u32 = {}; ", pipeline_layout_ref.id()));
    // writer.new_line();
    // // struct
    // writer.add_line(format!("pub struct {};", pipeline_layout.name));
    // writer.new_line();
    // // impl
    // writer.add_line(format!("impl {} {{", pipeline_layout.name));
    // writer.indent();
    // writer.add_line("pub fn id() -> u32 { ID }");
    // writer.unindent();
    // writer.add_line(format!("}} // {}", pipeline_layout.name));
    // writer.new_line();
    // // trait info
    // writer.add_line(format!(
    //     "impl CGenPipelineLayoutInfo for {} {{",
    //     pipeline_layout.name
    // ));
    // writer.indent();
    // writer.add_line("fn id() -> u32 { ID }");
    // writer.unindent();
    // writer.add_line("} // CGenPipelineLayoutInfo");
    // writer.new_line();

    // finalize
    writer.build()
}
