use heck::SnakeCase;

use crate::{
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
    model::{CGenType, DescriptorSet},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for descriptor_set in model.object_iter::<DescriptorSet>() {
        let content = generate_rust_descriptorset(&ctx, descriptor_set);
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

// fn from_descriptor(index: usize, descriptor: &Descriptor) -> graphics_api::DescriptorDef {
//     let shader_resource_type = match descriptor.def {
//         crate::model::DescriptorDef::Sampler => ShaderResourceType::Sampler,
//         crate::model::DescriptorDef::ConstantBuffer(_) => ShaderResourceType::ConstantBuffer,
//         crate::model::DescriptorDef::StructuredBuffer(_) => ShaderResourceType::StructuredBuffer,
//         crate::model::DescriptorDef::RWStructuredBuffer(_) => {
//             ShaderResourceType::RWStructuredBuffer
//         }
//         crate::model::DescriptorDef::ByteAddressBuffer => ShaderResourceType::ByteAdressBuffer,
//         crate::model::DescriptorDef::RWByteAddressBuffer => ShaderResourceType::RWByteAdressBuffer,
//         crate::model::DescriptorDef::Texture2D(_) => ShaderResourceType::Texture2D,
//         crate::model::DescriptorDef::RWTexture2D(_) => ShaderResourceType::RWTexture2D,
//         crate::model::DescriptorDef::Texture3D(_) => ShaderResourceType::Texture3D,
//         crate::model::DescriptorDef::RWTexture3D(_) => ShaderResourceType::RWTexture3D,
//         crate::model::DescriptorDef::Texture2DArray(_) => ShaderResourceType::Texture2DArray,
//         crate::model::DescriptorDef::RWTexture2DArray(_) => ShaderResourceType::RWTexture2DArray,
//         crate::model::DescriptorDef::TextureCube(_) => ShaderResourceType::TextureCube,
//         crate::model::DescriptorDef::TextureCubeArray(_) => ShaderResourceType::TextureCubeArray,
//     };

//     let array_size = if let Some(array_size) = descriptor.array_len {
//         array_size
//     } else {
//         0
//     };

//     let binding = u32::try_from(index).unwrap();

//     graphics_api::DescriptorDef {
//         name: descriptor.name.clone(),
//         binding,
//         shader_resource_type,
//         array_size,
//     }
// }

fn generate_rust_descriptorset(
    ctx: &GeneratorContext<'_>,
    descriptor_set: &DescriptorSet,
) -> String {
    let mut writer = FileWriter::new();

    // global dependencies
    writer.add_line("use lgn_graphics_api::DeviceContext;".to_string());
    writer.add_line("use lgn_graphics_api::DescriptorSetLayoutDef;".to_string());
    writer.add_line("use lgn_graphics_api::DescriptorSetLayout;".to_string());

    // local dependencies
    let deps = GeneratorContext::get_descriptorset_dependencies(descriptor_set);

    if !deps.is_empty() {
        for object_id in &deps {
            let dep_ty = ctx.model.get_from_objectid::<CGenType>(*object_id).unwrap();
            match dep_ty {
                CGenType::Native(_) => {}
                CGenType::Struct(e) => {
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
    writer.add_line("api_layout : DescriptorSetLayout,".to_string());
    writer.unindent();
    writer.add_line(format!("}}"));
    writer.new_line();
    // trait
    writer.add_line(format!("impl {} {{", descriptor_set.name));
    writer.indent();
    // new
    writer.add_line(format!(
        "pub fn new(device_context: &DeviceContext) -> Self {{"
    ));
    writer.indent();
    writer.add_line("let mut layout_def = DescriptorSetLayoutDef::default();".to_string());
    writer.add_line(format!(
        "layout_def.frequency = {};",
        descriptor_set.frequency
    ));
    for _descriptor_def in &descriptor_set.descriptors {}
    writer.add_line(
        "let api_layout = device_context.create_descriptorset_layout(&layout_def).unwrap();"
            .to_string(),
    );
    writer.add_line(format!("Self {{ api_layout }}"));
    writer.unindent();
    writer.add_line(format!("}}"));
    // api_layout
    writer.add_line(format!(
        "pub fn api_layout(&self) -> &DescriptorSetLayout {{ &self.api_layout }}"
    ));
    writer.unindent();
    writer.add_line(format!("}}"));

    // finalize
    writer.to_string()
}
