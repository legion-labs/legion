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

#[allow(clippy::too_many_lines)]
fn generate_rust_descriptorset(
    ctx: &GeneratorContext<'_>,
    descriptor_set_ref: DescriptorSetRef,
) -> String {
    let descriptor_set = descriptor_set_ref.get(ctx.model);

    let mut writer = FileWriter::new();

    // global dependencies
    writer.add_line("#[allow(unused_imports)]");
    writer.indent();
    writer.add_line("use lgn_graphics_api::{");
    writer.add_line("DeviceContext,");
    writer.add_line("DescriptorSetLayout,");
    writer.add_line("ShaderResourceType,");
    writer.add_line("DescriptorRef,");
    writer.add_line("Sampler,");
    writer.add_line("BufferView,");
    writer.add_line("TextureView,");
    writer.add_line("DescriptorSetDataProvider,");
    writer.unindent();
    writer.add_line("};");
    writer.new_line();

    writer.add_line("#[allow(unused_imports)]");
    writer.indent();
    writer.add_line("use lgn_graphics_cgen_runtime::{");
    writer.add_line("CGenDescriptorSetInfo,");
    writer.add_line("CGenDescriptorDef,");
    writer.add_line("CGenDescriptorSetDef,");
    writer.unindent();
    writer.add_line("};");
    writer.new_line();

    // write cgen descriptor def
    {
        writer.add_line(format!(
            "static DESCRIPTOR_DEFS: [CGenDescriptorDef; {}] = [",
            descriptor_set.descriptors.len()
        ));
        writer.indent();
        for descriptor in &descriptor_set.descriptors {
            writer.add_line("CGenDescriptorDef {");
            writer.indent();
            writer.add_line(format!("name: \"{}\",", descriptor.name));
            writer.add_line(format!(
                "shader_resource_type: ShaderResourceType::{},",
                descriptor.def.to_shader_resource_type()
            ));
            writer.add_line(format!("flat_index_start: {},", descriptor.flat_index));
            writer.add_line(format!(
                "flat_index_end: {},",
                descriptor.flat_index + descriptor.array_len.unwrap_or(1)
            ));
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
        writer.add_line("static DESCRIPTOR_SET_DEF: CGenDescriptorSetDef = CGenDescriptorSetDef{ ");
        writer.indent();
        writer.add_line(format!("name: \"{}\",", descriptor_set.name));
        writer.add_line(format!("id: {},", descriptor_set_ref.id()));
        writer.add_line(format!("frequency: {},", descriptor_set.frequency));
        writer.add_line(format!(
            "descriptor_flat_count: {},",
            descriptor_set.flat_descriptor_count
        ));
        writer.add_line("descriptor_defs: &DESCRIPTOR_DEFS,");
        writer.unindent();
        writer.add_line("}; ");
        writer.new_line();
    }

    // descriptor set layout
    {
        writer.add_line("static mut DESCRIPTOR_SET_LAYOUT: Option<DescriptorSetLayout> = None;");
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
        writer.add_line( "unsafe { DESCRIPTOR_SET_LAYOUT = Some(DESCRIPTOR_SET_DEF.create_descriptor_set_layout(device_context)); }" );
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // impl: shutdown
        writer.add_line("#[allow(unsafe_code)]");
        writer.add_line("pub fn shutdown() {");
        writer.indent();
        writer.add_line("unsafe{ DESCRIPTOR_SET_LAYOUT = None; }");
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        // impl: descriptor_set_layout
        writer.add_line("#[allow(unsafe_code)]");
        writer.add_line("pub fn descriptor_set_layout() -> &'static DescriptorSetLayout {");
        writer.indent();
        writer.add_line("unsafe{ match &DESCRIPTOR_SET_LAYOUT{");
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
        writer.add_line("pub fn def() -> &'static CGenDescriptorSetDef { &DESCRIPTOR_SET_DEF }");
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
                    (
                        0,
                        crate::model::DescriptorDef::ConstantBuffer(_)
                        | crate::model::DescriptorDef::StructuredBuffer(_)
                        | crate::model::DescriptorDef::RWStructuredBuffer(_)
                        | crate::model::DescriptorDef::ByteAddressBuffer
                        | crate::model::DescriptorDef::RWByteAddressBuffer,
                    ) => ("BufferView", "&'a BufferView".to_string()),
                    (
                        n,
                        crate::model::DescriptorDef::ConstantBuffer(_)
                        | crate::model::DescriptorDef::StructuredBuffer(_)
                        | crate::model::DescriptorDef::RWStructuredBuffer(_)
                        | crate::model::DescriptorDef::ByteAddressBuffer
                        | crate::model::DescriptorDef::RWByteAddressBuffer,
                    ) => ("BufferView", format!("&[&'a BufferView; {}]", n)),
                    (
                        0,
                        crate::model::DescriptorDef::Texture2D(_)
                        | crate::model::DescriptorDef::RWTexture2D(_)
                        | crate::model::DescriptorDef::Texture3D(_)
                        | crate::model::DescriptorDef::RWTexture3D(_)
                        | crate::model::DescriptorDef::Texture2DArray(_)
                        | crate::model::DescriptorDef::RWTexture2DArray(_)
                        | crate::model::DescriptorDef::TextureCube(_)
                        | crate::model::DescriptorDef::TextureCubeArray(_),
                    ) => ("TextureView", "&'a TextureView".to_string()),
                    (
                        n,
                        crate::model::DescriptorDef::Texture2D(_)
                        | crate::model::DescriptorDef::RWTexture2D(_)
                        | crate::model::DescriptorDef::Texture3D(_)
                        | crate::model::DescriptorDef::RWTexture3D(_)
                        | crate::model::DescriptorDef::Texture2DArray(_)
                        | crate::model::DescriptorDef::RWTexture2DArray(_)
                        | crate::model::DescriptorDef::TextureCube(_)
                        | crate::model::DescriptorDef::TextureCubeArray(_),
                    ) => ("TextureView", format!("&[&'a TextureView; {}]", n)),
                };

            writer.add_line(format!(
                "pub fn set_{}(&mut self, value:  {}) {{",
                descriptor.name, descriptor_input_decl
            ));
            writer.indent();

            if let Some(n) = descriptor.array_len {
                writer.add_line(format!(
                    "assert!(DESCRIPTOR_SET_DEF.descriptor_defs[{}].validate(&value.as_slice()));",
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
            } else {
                writer.add_line(format!(
                    "assert!(DESCRIPTOR_SET_DEF.descriptor_defs[{}].validate(value));",
                    descriptor_index
                ));
                writer.add_line(format!(
                    "self.descriptor_refs[{}] = DescriptorRef::{}(value);",
                    descriptor.flat_index, descriptor_ref_type
                ));
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
        writer.add_line("&self.descriptor_refs[
                DESCRIPTOR_DEFS[descriptor_index].flat_index_start as usize .. DESCRIPTOR_DEFS[descriptor_index].flat_index_end as usize
             ]".to_string());
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
