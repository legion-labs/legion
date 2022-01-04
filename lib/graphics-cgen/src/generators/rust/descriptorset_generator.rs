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

    writer.add_line("use lgn_graphics_api::DeviceContext;");
    writer.add_line("use lgn_graphics_api::DescriptorSetLayout;");
    writer.add_line("use lgn_graphics_api::ShaderResourceType;");
    writer.add_line("use lgn_graphics_api::DescriptorRef;");
    writer.add_line("use lgn_graphics_api::Sampler;");
    writer.add_line("use lgn_graphics_api::BufferView;");
    writer.add_line("use lgn_graphics_api::TextureView;");
    writer.add_line("use lgn_graphics_api::DescriptorSetDataProvider;");
    writer.add_line("use lgn_graphics_cgen_runtime::CGenDescriptorSetInfo;");
    writer.add_line("use lgn_graphics_cgen_runtime::CGenDescriptorDef;");
    writer.add_line("use lgn_graphics_cgen_runtime::CGenDescriptorSetDef;");

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

    // write cgen descriptor def
    {
        writer.add_line(format!(
            "static descriptor_defs: [CGenDescriptorDef; {}] = [",
            descriptor_set.descriptors.len()
        ));
        writer.indent();
        for descriptor in &descriptor_set.descriptors {
            writer.add_line("CGenDescriptorDef {");
            writer.indent();
            writer.add_line(format!("name: \"{}\",", descriptor.name));
            writer.add_line(format!(
                "shader_resource_type: ShaderResourceType::{},",
                descriptor.def.into_shader_resource_type()
            ));
            writer.add_line(format!("flat_index_start: {},", descriptor.flat_index));
            writer.add_line(format!("flat_index_end: {},", descriptor.flat_index + descriptor.array_len.unwrap_or(1) ));
            writer.add_line(format!(
                "array_size: {},",
                descriptor.array_len.unwrap_or(0u32)
            ));
            writer.unindent();
            writer.add_line("}, ");
        }
        writer.unindent();
        writer.add_line("];");
        writer.new_line();
    }

    // write cgen descriptor set def
    {
        writer.add_line("static descriptor_set_def: CGenDescriptorSetDef = CGenDescriptorSetDef{ ");
        writer.indent();
        writer.add_line(format!("name: \"{}\",", descriptor_set.name));
        writer.add_line(format!("id: {},", descriptor_set_ref.id()));
        writer.add_line(format!("frequency: {},", descriptor_set.frequency));
        writer.add_line(format!(
            "descriptor_flat_count: {},",
            descriptor_set.flat_descriptor_count
        ));
        writer.add_line("descriptor_defs: &descriptor_defs,");
        writer.unindent();
        writer.add_line("}; ");
        writer.new_line();
    }

    // descriptor set layout
    {
        writer.add_line("static mut descriptor_set_layout: Option<DescriptorSetLayout> = None;");
        writer.new_line();
    }

    // struct
    {
        writer.add_line(format!("pub struct {}<'a> {{", descriptor_set.name));
        writer.indent();
        writer.add_line(format!(
            "descriptor_refs: [DescriptorRef<'a>; {}],",
            descriptor_set.flat_descriptor_count
        ));
        writer.unindent();
        writer.add_line("}");
        writer.new_line();
    }

    // impl
    {
        writer.add_line(format!("impl<'a> {}<'a> {{", descriptor_set.name));
        writer.indent();
        writer.new_line();

        // impl: initialize
        writer.add_line("#[allow(unsafe_code)]");
        writer.add_line("pub fn initialize(device_context: &DeviceContext) {");
        writer.indent();
        writer.add_line( "unsafe { descriptor_set_layout = Some(descriptor_set_def.create_descriptor_set_layout(device_context)); }" );
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // impl: shutdown
        writer.add_line("#[allow(unsafe_code)]");        
        writer.add_line("pub fn shutdown() {");
        writer.indent();
        writer.add_line( "unsafe{ descriptor_set_layout = None; }" );
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // impl: descriptor_set_layout
        writer.add_line("#[allow(unsafe_code)]");
        writer.add_line("pub fn descriptor_set_layout() -> &'static DescriptorSetLayout {");
        writer.indent();
        writer.add_line("unsafe{ match &descriptor_set_layout{");
        writer.add_line("Some(dsl) => dsl,");
        writer.add_line("None => unreachable!(),");
        writer.add_line("}}");
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // impl: id
        writer.add_line(format!(
            "pub const fn id() -> u32 {{ {}  }}",
            descriptor_set_ref.id()
        ));
        writer.new_line();

        // impl: frequency
        writer.add_line(format!(
            "pub const fn frequency() -> u32 {{ {}  }}",
            descriptor_set.frequency
        ));
        writer.new_line();

        // impl: def
        writer.add_line("pub fn def() -> &'static CGenDescriptorSetDef { &descriptor_set_def }");
        writer.new_line();

        // impl: new
        writer.add_line("pub fn new() -> Self { Self::default() }");
        writer.new_line();

        // impl: set methods
        for (descriptor_index, descriptor) in descriptor_set.descriptors.iter().enumerate() {
            let (descriptor_ref_type, descriptor_input_decl) =
                match (descriptor.array_len.unwrap_or(0u32), &descriptor.def) {
                    (0, crate::model::DescriptorDef::Sampler) => {
                        ("Sampler", "&'a Sampler".to_string())
                    }
                    (n, crate::model::DescriptorDef::Sampler) => {
                        ("Sampler", format!("&[&'a Sampler; {}]", n))
                    }
                    (0, crate::model::DescriptorDef::ConstantBuffer(_))
                    | (0, crate::model::DescriptorDef::StructuredBuffer(_))
                    | (0, crate::model::DescriptorDef::RWStructuredBuffer(_))
                    | (0, crate::model::DescriptorDef::ByteAddressBuffer)
                    | (0, crate::model::DescriptorDef::RWByteAddressBuffer) => {
                        ("BufferView", "&'a BufferView".to_string())
                    }
                    (n, crate::model::DescriptorDef::ConstantBuffer(_))
                    | (n, crate::model::DescriptorDef::StructuredBuffer(_))
                    | (n, crate::model::DescriptorDef::RWStructuredBuffer(_))
                    | (n, crate::model::DescriptorDef::ByteAddressBuffer)
                    | (n, crate::model::DescriptorDef::RWByteAddressBuffer) => {
                        ("BufferView", format!("&[&'a BufferView; {}]", n))
                    }
                    (0, crate::model::DescriptorDef::Texture2D(_))
                    | (0, crate::model::DescriptorDef::RWTexture2D(_))
                    | (0, crate::model::DescriptorDef::Texture3D(_))
                    | (0, crate::model::DescriptorDef::RWTexture3D(_))
                    | (0, crate::model::DescriptorDef::Texture2DArray(_))
                    | (0, crate::model::DescriptorDef::RWTexture2DArray(_))
                    | (0, crate::model::DescriptorDef::TextureCube(_))
                    | (0, crate::model::DescriptorDef::TextureCubeArray(_)) => {
                        ("TextureView", "&'a TextureView".to_string())
                    }
                    (n, crate::model::DescriptorDef::Texture2D(_))
                    | (n, crate::model::DescriptorDef::RWTexture2D(_))
                    | (n, crate::model::DescriptorDef::Texture3D(_))
                    | (n, crate::model::DescriptorDef::RWTexture3D(_))
                    | (n, crate::model::DescriptorDef::Texture2DArray(_))
                    | (n, crate::model::DescriptorDef::RWTexture2DArray(_))
                    | (n, crate::model::DescriptorDef::TextureCube(_))
                    | (n, crate::model::DescriptorDef::TextureCubeArray(_)) => {
                        ("TextureView", format!("&[&'a TextureView; {}]", n))
                    }
                };

            writer.add_line(format!(
                "pub fn set_{}(&mut self, value:  {}) {{",
                descriptor.name, descriptor_input_decl
            ));
            writer.indent();

            match descriptor.array_len {
                Some(n) => {
                    writer.add_line(format!(
                        "assert!(descriptor_set_def.descriptor_defs[{}].validate(value.as_ref()));",
                        descriptor_index
                    ));
                    writer.add_line(format!("for i in 0..{} {{", n));
                    writer.indent();
                    writer.add_line(format!(
                        "self.descriptor_refs[{}+i] = DescriptorRef::{}(value[i]);",
                        descriptor.flat_index, descriptor_ref_type
                    ));
                    writer.unindent();
                    writer.add_line("}");
                }
                None => {
                    writer.add_line(format!(
                        "assert!(descriptor_set_def.descriptor_defs[{}].validate(value));",
                        descriptor_index
                    ));
                    writer.add_line(format!(
                        "self.descriptor_refs[{}] = DescriptorRef::{}(value);",
                        descriptor.flat_index, descriptor_ref_type
                    ));
                }
            }

            writer.unindent();
            writer.add_line("}");
            writer.new_line();
        }

        writer.unindent();
        writer.add_line("}");
        writer.new_line();
    }

    // trait: default
    {
        writer.add_line(format!(
            "impl<'a> Default for {}<'a> {{",
            descriptor_set.name
        ));
        writer.indent();
        writer.add_line("fn default() -> Self {");
        writer.indent();
        writer.add_line(format!(
            "Self {{descriptor_refs: [DescriptorRef::<'a>::default(); {}], }}",
            descriptor_set.flat_descriptor_count
        ));
        writer.unindent();
        writer.add_line("}");
        writer.unindent();
        writer.add_line("}");
        writer.new_line();
    }

    // trait: DescriptorSetDataProvider
    {
        writer.add_line(format!(
            "impl<'a> DescriptorSetDataProvider for {}<'a> {{",
            descriptor_set.name
        ));
        writer.indent();

        writer.add_line("fn layout(&self) -> &'static DescriptorSetLayout {");
        writer.indent();
        writer.add_line("Self::descriptor_set_layout()");
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        writer.add_line(
            "fn descriptor_refs(&self, descriptor_index: usize) -> &[DescriptorRef<'a>] {",
        );
        writer.indent();
        writer.add_line(format!(
            "&self.descriptor_refs[
                descriptor_defs[descriptor_index].flat_index_start as usize .. descriptor_defs[descriptor_index].flat_index_end as usize
             ]",            
        ));
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        writer.unindent();
        writer.add_line("}");
        writer.new_line();
    }

    // finalize
    writer.build()
}
