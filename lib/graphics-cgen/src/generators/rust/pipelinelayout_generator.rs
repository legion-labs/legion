use lgn_graphics_api::MAX_DESCRIPTOR_SET_LAYOUTS;

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

#[allow(clippy::too_many_lines)]
fn generate_rust_pipeline_layout(
    ctx: &GeneratorContext<'_>,
    pipeline_layout_ref: PipelineLayoutRef,
) -> String {
    let pipeline_layout = pipeline_layout_ref.get(ctx.model);

    let mut writer = FileWriter::new();

    // global dependencies
    writer.add_line("use std::{mem, ptr};");
    writer.new_line();

    writer.add_line("use lgn_graphics_api::{");
    writer.indent();
    writer.add_line("DeviceContext,");
    writer.add_line("RootSignature,");
    writer.add_line("DescriptorSetLayout,");
    writer.add_line("DescriptorSetHandle,");
    writer.add_line("Pipeline,");
    writer.add_line("MAX_DESCRIPTOR_SET_LAYOUTS,");
    writer.unindent();
    writer.add_line("};");
    writer.new_line();

    writer.add_line("use lgn_graphics_cgen_runtime::{");
    writer.indent();
    writer.add_line("CGenPipelineLayoutDef,");
    writer.add_line("PipelineDataProvider,");
    writer.unindent();
    writer.add_line("};");
    writer.new_line();

    // local dependencies
    {
        for (_, content) in &pipeline_layout.members {
            match content {
                crate::model::PipelineLayoutContent::DescriptorSet(ds_ref) => {
                    let ds = ds_ref.get(ctx.model);
                    writer.add_line(format!("use super::super::descriptor_set::{};", ds.name));
                }
                crate::model::PipelineLayoutContent::Pushconstant(ty_ref) => {
                    let ty = ty_ref.get(ctx.model);
                    writer.add_line(format!("use super::super::cgen_type::{};", ty.name()));
                }
            }
        }
    }
    writer.new_line();

    // write cgen pipeline layout def
    {
        writer.add_line(
            "static PIPELINE_LAYOUT_DEF: CGenPipelineLayoutDef = CGenPipelineLayoutDef{ ",
        );
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
        if let Some(ty_ref) = pipeline_layout.push_constant() {
            let ty = ty_ref.get(ctx.model);
            writer.add_line(format!("push_constant_type: Some({}::id())", ty.name()));
        } else {
            writer.add_line("push_constant_type: None,");
        }

        writer.unindent();
        writer.add_line("}; ");
        writer.new_line();
    }

    // pipeline layout
    {
        writer.add_line("static mut PIPELINE_LAYOUT: Option<RootSignature> = None;");
        writer.new_line();
    }

    // struct
    {
        writer.add_line(format!("pub struct {}<'a> {{", pipeline_layout.name));
        writer.indent();
        writer.add_line("pipeline: &'a Pipeline,");
        writer.add_line(
            "descriptor_sets: [Option<DescriptorSetHandle>; MAX_DESCRIPTOR_SET_LAYOUTS],",
        );
        if let Some(ty_ref) = pipeline_layout.push_constant() {
            let ty = ty_ref.get(ctx.model);
            writer.add_line(format!("push_constant: {}", ty.name()));
        }
        writer.unindent();
        writer.add_line("}");
        writer.new_line();
    }

    // impl
    {
        writer.add_line(format!("impl<'a> {}<'a> {{", pipeline_layout.name));
        writer.indent();
        writer.new_line();

        // fn initialize
        writer.add_line("#[allow(unsafe_code)]");
        writer.add_line("pub fn initialize(device_context: &DeviceContext, descriptor_set_layouts: &[&DescriptorSetLayout]) {");
        writer.indent();
        writer.add_line("unsafe { ");
        if let Some(ty_ref) = pipeline_layout.push_constant() {
            let ty = ty_ref.get(ctx.model);
            writer.add_line(format!(
                "let push_constant_def = Some({}::def());",
                ty.name()
            ));
        } else {
            writer.add_line("let push_constant_def = None");
        };
        writer.add_line( "PIPELINE_LAYOUT = Some(PIPELINE_LAYOUT_DEF.create_pipeline_layout(device_context, descriptor_set_layouts, push_constant_def));" );
        writer.add_line("}");
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // fn shutdown
        writer.add_line("#[allow(unsafe_code)]");
        writer.add_line("pub fn shutdown() {");
        writer.indent();
        writer.add_line("unsafe{ PIPELINE_LAYOUT = None; }");
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // fn root_signature
        writer.add_line("#[allow(unsafe_code)]");
        writer.add_line("pub fn root_signature() -> &'static RootSignature {");
        writer.indent();
        writer.add_line("unsafe{ match &PIPELINE_LAYOUT{");
        writer.indent();
        writer.add_line("Some(pl) => pl,");
        writer.add_line("None => unreachable!(),");
        writer.unindent();
        writer.add_line("}}");
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // fn new
        writer.add_line("pub fn new(pipeline: &'a Pipeline) -> Self {");
        writer.indent();
        writer.add_line("assert_eq!( pipeline.root_signature(), Self::root_signature());");
        writer.add_line("Self {");
        writer.indent();
        writer.add_line("pipeline,");
        writer.add_line("descriptor_sets: [None; MAX_DESCRIPTOR_SET_LAYOUTS],");
        if let Some(ty_ref) = pipeline_layout.push_constant() {
            let ty = ty_ref.get(ctx.model);
            writer.add_line(format!("push_constant: {}::default(),", ty.name()));
        }
        writer.unindent();
        writer.add_line("}");
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // fn setters
        for (name, content) in &pipeline_layout.members {
            match content {
                crate::model::PipelineLayoutContent::DescriptorSet(ds_ref) => {
                    let ds = ds_ref.get(ctx.model);
                    writer.add_line(format!(
                        "pub fn set_{}(&mut self, descriptor_set_handle: DescriptorSetHandle) {{",
                        name
                    ));
                    writer.indent();
                    writer.add_line(format!(
                        "self.descriptor_sets[{}] = Some(descriptor_set_handle);",
                        ds.frequency
                    ));
                    writer.unindent();
                    writer.add_line("}");
                }
                crate::model::PipelineLayoutContent::Pushconstant(ty_ref) => {
                    let ty = ty_ref.get(ctx.model);
                    writer.add_line(format!(
                        "pub fn set_{}(&mut self, data: &{}) {{",
                        name,
                        ty.name()
                    ));
                    writer.indent();
                    writer.add_line("self.push_constant = *data;");
                    writer.unindent();
                    writer.add_line("}");
                }
            }
        }

        // prolog
        writer.unindent();
        writer.add_line("}");
        writer.new_line();
    }

    // trait: PipelineDataProvider
    {
        writer.add_line(format!(
            "impl<'a> PipelineDataProvider for {}<'a> {{",
            pipeline_layout.name
        ));
        writer.indent();
        writer.new_line();

        // fn pipeline
        writer.add_line("fn pipeline(&self) -> &Pipeline {");
        writer.indent();
        writer.add_line("self.pipeline");
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // fn descriptor_set
        writer
            .add_line("fn descriptor_set(&self, frequency: u32) -> Option<DescriptorSetHandle> {");
        writer.indent();
        writer.add_line("self.descriptor_sets[frequency as usize]");
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // fn push_constant
        writer.add_line("fn push_constant(&self) -> Option<&[u8]> {");
        writer.indent();
        if let Some(ty_ref) = pipeline_layout.push_constant() {
            writer.add_line("#![allow(unsafe_code)]");
            let ty = ty_ref.get(ctx.model);
            writer.add_line("let data_slice = unsafe {");
            writer.add_line(format!("&*ptr::slice_from_raw_parts((&self.push_constant as *const {0}).cast::<u8>(), mem::size_of::<{0}>())", ty.name()));
            writer.add_line("};");
            writer.add_line("Some(data_slice)");
        } else {
            writer.add_line("None");
        }
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // fn set_descriptor_set
        writer.add_line("fn set_descriptor_set(&mut self, frequency: u32, descriptor_set: Option<DescriptorSetHandle>) {" );
        writer.indent();
        writer.add_line("self.descriptor_sets[frequency as usize] = descriptor_set;");
        writer.unindent();
        writer.add_line("}");

        // prolog
        writer.unindent();
        writer.add_line("}");
        writer.new_line();
    }

    // finalize
    writer.build()
}
