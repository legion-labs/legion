use relative_path::RelativePath;

use crate::{
    db::{CGenType, DescriptorSet, Model, ModelObject, PipelineLayout},
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let content = generate(ctx);
    products.push(Product::new(
        CGenVariant::Rust,
        RelativePath::new("mod.rs").to_relative_path_buf(),
        content.into_bytes(),
    ));

    products
}

fn write_mod<T>(model: &Model, writer: &mut FileWriter)
where
    T: ModelObject,
{
    if model.size::<T>() > 0 {
        let folder = GeneratorContext::get_object_folder::<T>();
        writer.add_line(format!("pub mod {};", folder));
    }
}

#[rustfmt::skip]
fn generate(ctx: &GeneratorContext<'_>) -> String {
    let mut writer = FileWriter::new();

    // write lints disabling
    writer.add_line("#![allow(clippy::all)]");
    writer.add_line("#![allow(dead_code)]");
    writer.new_line();

    // write dependencies
    let model = ctx.model;    
    writer.add_line("use lgn_graphics_api::DeviceContext;");
    write_mod::<CGenType>(model, &mut writer);
    write_mod::<DescriptorSet>(model, &mut writer);
    write_mod::<PipelineLayout>(model, &mut writer);
    writer.new_line();    

    // fn initialize
    {
        writer.add_line( "pub fn initialize(device_context: &DeviceContext) {" );
        writer.indent();
           
        for descriptor_set_ref in model.object_iter::<DescriptorSet>() {
            writer.add_line( format!("descriptor_set::{}::initialize(device_context);", descriptor_set_ref.object().name));            
        }
        
        writer.new_line();
        writer.add_line("let descriptor_set_layouts = [");
        writer.indent();
        for descriptor_set_ref in model.object_iter::<DescriptorSet>() {
            writer.add_line( format!("descriptor_set::{}::descriptor_set_layout(),", descriptor_set_ref.object().name));            
        }
        writer.unindent();
        writer.add_line("];");
        
        writer.new_line();
        for pipeline_layout_ref in model.object_iter::<PipelineLayout>() {
            writer.add_line( format!("pipeline_layout::{}::initialize(device_context, &descriptor_set_layouts);", pipeline_layout_ref.object().name));    
        }    
        writer.unindent();
        writer.add_line( "}" );
        writer.new_line();
    }

    // fn shutdown
    {
        writer.add_line( "pub fn shutdown() {" );
        writer.indent();
    
        for descriptor_set_ref in model.object_iter::<DescriptorSet>() {
            writer.add_line( format!("descriptor_set::{}::shutdown();", descriptor_set_ref.object().name));            
        }
        writer.new_line();
        
        for pipeline_layout_ref in model.object_iter::<PipelineLayout>() {
            writer.add_line( format!("pipeline_layout::{}::shutdown();", pipeline_layout_ref.object().name));    
        }    
        writer.unindent();
        writer.add_line( "}" );   
    }
    
    writer.build()
}
