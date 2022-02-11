use crate::{
    db::Shader,
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for sh_ref in model.object_iter::<Shader>() {
        let content = generate_rust_shader(ctx, sh_ref.id(), sh_ref.object());
        products.push(Product::new(
            CGenVariant::Rust,
            GeneratorContext::object_relative_path(sh_ref.object(), CGenVariant::Rust),
            content.into_bytes(),
        ));
    }

    products
}

fn generate_rust_shader(ctx: &GeneratorContext<'_>, sh_id: u32, shader: &Shader) -> String {
    let mut writer = FileWriter::new();

    

    writer.add_lines( 
        &[
        "use super::super::CRATE_ID;",
        "use lgn_embedded_fs::embedded_watched_file;",
        "use lgn_graphics_api::ShaderStageFlags;",
        "use lgn_graphics_cgen_runtime::{",
        "    CGenShaderFamily, CGenShaderFamilyID, CGenShaderInstance, CGenShaderKey, CGenShaderOption,",
        "};"
        ]
    );
    writer.new_line();

    writer.add_line( format!("embedded_watched_file!(SHADER_PATH, \"{}\");", shader.path) );
    writer.new_line();

    writer.add_line( format!("pub const ID: CGenShaderFamilyID = CGenShaderFamilyID::make(CRATE_ID, {});", sh_id ) );
    writer.new_line();


    writer.add_line( "pub const NONE: u64 = 0;" );
    for (i, option) in shader.options.iter().enumerate() {
        writer.add_line( format!("pub const {option}: u64 = 1u64 << {i};" ) );
    }
    writer.new_line();

    {
        let mut writer = writer.add_block(
            &[&format!("pub static SHADER_OPTIONS: [CGenShaderOption; {}] = [", shader.options.len())], 
            &["];"]
        );        
        
        for (i, option) in shader.options.iter().enumerate() {
            writer.add_line( "CGenShaderOption {" );
            writer.add_line( format!("index: {i},") );
            writer.add_line( format!("name: \"{option}\",") );
            writer.add_line( "}," );
        }
    }   
    writer.new_line();
    {
        let mut writer = writer.add_block(
            &[&format!("pub static SHADER_INSTANCES: [CGenShaderInstance; {}] = [", shader.instances.len())], 
            &["];"]
        );        
        
        for (i, option) in shader.instances.iter().enumerate() {
            writer.add_line( "CGenShaderInstance {" );
            writer.add_line( format!("key: CGenShaderKey::make(ID, {}),", key_list ) );
            writer.add_line( format!("key: CGenShaderKey::make(ID, {}),", key_list ) );
            writer.add_line( "}," );
        }

    }
    writer.new_line();

    writer.add_line("pub static SHADER_FAMILY: CGenShaderFamily = CGenShaderFamily {");
    writer.add_line("    id: ID,");
    writer.add_line("    name: \"EGui\",");
    writer.add_line("    path: SHADER_PATH.path(),");
    writer.add_line("    options: &SHADER_OPTIONS,");
    writer.add_line("    instances: &SHADER_INSTANCES,");
    writer.add_line("};");

    writer.build()
}

/*
fn generate_rust_descriptor_set(
    _ctx: &GeneratorContext<'_>,
    descriptor_set_id: u32,
    descriptor_set: &DescriptorSet,
) -> String {
    let mut writer = FileWriter::new();

    // global dependencies
    {
        let mut writer = writer.add_block(
            &["#[allow(unused_imports)]", "use lgn_graphics_api::{"],
            &["};"],
        );
        writer.add_lines(&[
            "DeviceContext,",
            "DescriptorSetLayout,",
            "ShaderResourceType,",
            "DescriptorRef,",
            "Sampler,",
            "BufferView,",
            "TextureView,",
            "DescriptorSetDataProvider,",
        ]);
    }
    writer.new_line();
    {
        let mut writer = writer.add_block(
            &[
                "#[allow(unused_imports)]",
                "use lgn_graphics_cgen_runtime::{",
            ],
            &["};"],
        );
        writer.add_lines(&["CGenDescriptorDef,", "CGenDescriptorSetDef,"]);
    }
    writer.new_line();

    // write cgen descriptor def
    {
        let mut writer = writer.add_block(
            &[format!(
                "static DESCRIPTOR_DEFS: [CGenDescriptorDef; {}] = [",
                descriptor_set.descriptors.len()
            )],
            &["];"],
        );
        for descriptor in &descriptor_set.descriptors {
            let mut writer = writer.add_block(&["CGenDescriptorDef {"], &["},"]);
            writer.add_line(format!("name: \"{}\",", descriptor.name));
            writer.add_line(format!(
                "shader_resource_type: ShaderResourceType::{},",
                descriptor.def.shader_resource_type()
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
        }
    }
    writer.new_line();

    // write cgen descriptor set def
    {
        let mut writer = writer.add_block(
            &["static DESCRIPTOR_SET_DEF: CGenDescriptorSetDef = CGenDescriptorSetDef{"],
            &["};"],
        );
        writer.add_lines(&[
            format!("name: \"{}\",", descriptor_set.name),
            format!("id: {},", descriptor_set_id),
            format!("frequency: {},", descriptor_set.frequency),
            format!(
                "descriptor_flat_count: {},",
                descriptor_set.flat_descriptor_count
            ),
            "descriptor_defs: &DESCRIPTOR_DEFS,".to_string(),
        ]);
    }
    writer.new_line();
    // descriptor set layout
    {
        writer.add_line("static mut DESCRIPTOR_SET_LAYOUT: Option<DescriptorSetLayout> = None;");
        writer.new_line();
    }

    // struct
    {
        let mut writer = writer.add_block(
            &[format!("pub struct {}<'a> {{", descriptor_set.name)],
            &["}"],
        );

        writer.add_line(format!(
            "descriptor_refs: [DescriptorRef<'a>; {}],",
            descriptor_set.flat_descriptor_count
        ));
    }

    writer.new_line();

    // impl
    {
        let mut writer = writer.add_block(
            &[format!("impl<'a> {}<'a> {{", descriptor_set.name)],
            &["}"],
        );

        // impl: initialize
        {
            let mut writer = writer.add_block(
                &[
                    "#[allow(unsafe_code)]",
                    "pub fn initialize(descriptor_set_layout: &DescriptorSetLayout) {",
                ],
                &["}"],
            );
            writer.add_line(
                "unsafe { DESCRIPTOR_SET_LAYOUT = Some(descriptor_set_layout.clone()); }",
            );
        }

        writer.new_line();

        // impl: shutdown
        {
            let mut writer =
                writer.add_block(&["#[allow(unsafe_code)]", "pub fn shutdown() {"], &["}"]);
            writer.add_line("unsafe{ DESCRIPTOR_SET_LAYOUT = None; }");
        }

        writer.new_line();

        // impl: descriptor_set_layout
        {
            let mut writer = writer.add_block(
                &[
                    "#[allow(unsafe_code)]",
                    "pub fn descriptor_set_layout() -> &'static DescriptorSetLayout {",
                ],
                &["}"],
            );
            writer.add_line("unsafe{ match &DESCRIPTOR_SET_LAYOUT{");
            writer.add_line("Some(dsl) => dsl,");
            writer.add_line("None => unreachable!(),");
            writer.add_line("}}");
        }
        writer.new_line();

        // impl: id
        writer.add_line(format!(
            "pub const fn id() -> u32 {{ {}  }}",
            descriptor_set_id
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
                    (0, crate::db::DescriptorDef::Sampler) => {
                        ("Sampler", "&'a Sampler".to_string())
                    }
                    (n, crate::db::DescriptorDef::Sampler) => {
                        ("Sampler", format!("&[&'a Sampler; {}]", n))
                    }
                    (
                        0,
                        crate::db::DescriptorDef::ConstantBuffer(_)
                        | crate::db::DescriptorDef::StructuredBuffer(_)
                        | crate::db::DescriptorDef::RWStructuredBuffer(_)
                        | crate::db::DescriptorDef::ByteAddressBuffer
                        | crate::db::DescriptorDef::RWByteAddressBuffer,
                    ) => ("BufferView", "&'a BufferView".to_string()),
                    (
                        n,
                        crate::db::DescriptorDef::ConstantBuffer(_)
                        | crate::db::DescriptorDef::StructuredBuffer(_)
                        | crate::db::DescriptorDef::RWStructuredBuffer(_)
                        | crate::db::DescriptorDef::ByteAddressBuffer
                        | crate::db::DescriptorDef::RWByteAddressBuffer,
                    ) => ("BufferView", format!("&[&'a BufferView; {}]", n)),
                    (
                        0,
                        crate::db::DescriptorDef::Texture2D(_)
                        | crate::db::DescriptorDef::RWTexture2D(_)
                        | crate::db::DescriptorDef::Texture3D(_)
                        | crate::db::DescriptorDef::RWTexture3D(_)
                        | crate::db::DescriptorDef::Texture2DArray(_)
                        | crate::db::DescriptorDef::RWTexture2DArray(_)
                        | crate::db::DescriptorDef::TextureCube(_)
                        | crate::db::DescriptorDef::TextureCubeArray(_),
                    ) => ("TextureView", "&'a TextureView".to_string()),
                    (
                        n,
                        crate::db::DescriptorDef::Texture2D(_)
                        | crate::db::DescriptorDef::RWTexture2D(_)
                        | crate::db::DescriptorDef::Texture3D(_)
                        | crate::db::DescriptorDef::RWTexture3D(_)
                        | crate::db::DescriptorDef::Texture2DArray(_)
                        | crate::db::DescriptorDef::RWTexture2DArray(_)
                        | crate::db::DescriptorDef::TextureCube(_)
                        | crate::db::DescriptorDef::TextureCubeArray(_),
                    ) => ("TextureView", format!("&[&'a TextureView; {}]", n)),
                };

            {
                let mut writer = writer.add_block(
                    &[format!(
                        "pub fn set_{}(&mut self, value:  {}) {{",
                        descriptor.name, descriptor_input_decl
                    )],
                    &["}"],
                );
                if let Some(n) = descriptor.array_len {
                    writer.add_line(format!(
                        "assert!(DESCRIPTOR_SET_DEF.descriptor_defs[{}].validate(&value.as_slice()));",
                        descriptor_index
                    ));
                    {
                        let mut writer =
                            writer.add_block(&[format!("for i in 0..{} {{", n)], &["}"]);
                        writer.add_line(format!(
                            "self.descriptor_refs[{}+i] = DescriptorRef::{}(value[i]);",
                            descriptor.flat_index, descriptor_ref_type
                        ));
                    }
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
            }
            writer.new_line();
        }
        writer.new_line();
    }

    writer.new_line();

    // trait: default
    {
        let mut writer = writer.add_block(
            &[format!(
                "impl<'a> Default for {}<'a> {{",
                descriptor_set.name
            )],
            &["}"],
        );
        writer.add_line("fn default() -> Self {");
        {
            let _writer = writer.add_block(
                &[format!(
                    "Self {{descriptor_refs: [DescriptorRef::<'a>::default(); {}], }}",
                    descriptor_set.flat_descriptor_count
                )],
                &["}"],
            );
        }
    }

    writer.new_line();

    // trait: DescriptorSetDataProvider
    {
        let mut writer = writer.add_block(
            &[format!(
                "impl<'a> DescriptorSetDataProvider for {}<'a> {{",
                descriptor_set.name
            )],
            &["}"],
        );

        {
            let mut writer = writer.add_block(&["fn frequency(&self) -> u32 {"], &["}"]);
            writer.add_line("Self::frequency()");
        }

        writer.new_line();

        {
            let mut writer = writer.add_block(
                &["fn layout(&self) -> &'static DescriptorSetLayout {"],
                &["}"],
            );
            writer.add_line("Self::descriptor_set_layout()");
        }
        writer.new_line();

        {
            let mut writer = writer.add_block(
                &["fn descriptor_refs(&self, descriptor_index: usize) -> &[DescriptorRef<'a>] {"],
                &["}"],
            );
            writer.add_line("&self.descriptor_refs[
                DESCRIPTOR_DEFS[descriptor_index].flat_index_start as usize .. DESCRIPTOR_DEFS[descriptor_index].flat_index_end as usize
             ]");
        }
    }

    writer.new_line();
    // finalize
    writer.build()
}
 */
