use heck::ToSnakeCase;
use relative_path::RelativePath;

use crate::{
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
    model::{CGenType, DescriptorSet, Model, ModelObject, PipelineLayout},
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
        writer.add_line(format!("pub(crate) mod {};", folder.to_string()));
        writer.add_line("#[allow(unused_imports)]");
        writer.add_line(format!("pub(crate) use {}::*;", folder.to_string()));
    }
}

#[rustfmt::skip]
fn generate(ctx: &GeneratorContext<'_>) -> String {
    let mut writer = FileWriter::new();

    // write dependencies
    let model = ctx.model;
    writer.add_line("use lgn_graphics_api::DeviceContext;");
    write_mod::<CGenType>(model, &mut writer);
    write_mod::<DescriptorSet>(model, &mut writer);
    write_mod::<PipelineLayout>(model, &mut writer);

    // write struct
    writer.new_line();    
    writer.add_line( "pub struct CodeGen {" );
    writer.indent();
        for descriptor_set in model.object_iter::<DescriptorSet>() {
            writer.add_line( format!("{}: {},", descriptor_set.name.to_snake_case(), descriptor_set.name));    
        }
    writer.unindent();
    writer.add_line( "}" );
    writer.new_line();    
    // write trait
    writer.add_line( "impl CodeGen {") ;
        writer.indent();
        // write new
        writer.add_line( "pub fn new(device_context: &DeviceContext) -> Self {" );    
            writer.indent();
            writer.add_line( "Self{");
                writer.indent();
                for descriptor_set in model.object_iter::<DescriptorSet>() {
                    writer.add_line( format!("{}: {}::new(device_context), ", descriptor_set.name.to_snake_case(), descriptor_set.name));    
                }
                writer.unindent();
            writer.add_line( "}");
            writer.unindent();
        writer.add_line( "}");
        // write accessors
        for descriptor_set in model.object_iter::<DescriptorSet>() { 
            writer.add_line( format!("pub fn {}(&self) -> &{} {{ &self.{}  }}", descriptor_set.name.to_snake_case(), descriptor_set.name, descriptor_set.name.to_snake_case()));    
        }
        //...
        writer.unindent();
    writer.add_line( "}");
    writer.new_line();

    writer.build()
}

// struct DescriptorDef {}

// struct DescriptorSetLayoutDef {
//     frequency: u32,
//     descriptor_count: u32,
//     descriptor_defs: [DescriptorDef; 64],
// }

// struct FrameDescriptorSetLayout {
//     layout_def: DescriptorSetLayoutDef,
// }
