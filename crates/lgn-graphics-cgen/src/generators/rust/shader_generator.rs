use lgn_graphics_api::ShaderStage;

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

fn generate_rust_shader(_ctx: &GeneratorContext<'_>, sh_id: u32, shader: &Shader) -> String {
    let mut writer = FileWriter::new();

    writer.add_lines(&[
        "use lgn_embedded_fs::embedded_watched_file;",
        "use lgn_graphics_api::ShaderStageFlags;",
        "use lgn_graphics_cgen_runtime::{",
        "    CGenShaderDef, CGenShaderID, CGenShaderInstance, CGenShaderKey, CGenShaderOption,",
        "};",
    ]);
    writer.new_line();

    writer.add_line(format!(
        "embedded_watched_file!(SHADER_PATH, \"{}\");",
        shader.path
    ));
    writer.new_line();

    writer.add_line(format!(
        "pub const ID: CGenShaderID = CGenShaderID::make({});",
        sh_id
    ));
    writer.new_line();

    writer.add_line("pub const NONE: u64 = 0;");
    for (i, option) in shader.options.iter().enumerate() {
        writer.add_line(format!("pub const {option}: u64 = 1u64 << {i};"));
    }
    writer.new_line();

    {
        let mut writer = writer.add_block(
            &[&format!(
                "pub static SHADER_OPTIONS: [CGenShaderOption; {}] = [",
                shader.options.len()
            )],
            &["];"],
        );

        for (i, option) in shader.options.iter().enumerate() {
            writer.add_line("CGenShaderOption {");
            writer.add_line(format!("index: {i},"));
            writer.add_line(format!("name: \"{option}\","));
            writer.add_line("},");
        }
    }
    writer.new_line();
    {
        let mut writer = writer.add_block(
            &[&format!(
                "pub static SHADER_INSTANCES: [CGenShaderInstance; {}] = [",
                shader.instances.len()
            )],
            &["];"],
        );

        for (_, instance) in shader.instances.iter().enumerate() {
            let mut key_list = instance
                .keys
                .iter()
                .map(|x| shader.options[*x as usize].as_str())
                .collect::<Vec<&str>>()
                .join("|");
            if key_list.is_empty() {
                key_list = "NONE".to_owned();
            }
            let mut stage_flags = "ShaderStageFlags::from_bits_truncate(".to_string();

            stage_flags.push_str(
                instance
                    .stages
                    .iter()
                    .map(|s| match s {
                        ShaderStage::Vertex => "ShaderStageFlags::VERTEX_FLAG.bits()",
                        ShaderStage::Fragment => "ShaderStageFlags::FRAGMENT_FLAG.bits()",
                        ShaderStage::Compute => "ShaderStageFlags::COMPUTE_FLAG.bits()",
                    })
                    .collect::<Vec<_>>()
                    .join("|")
                    .as_str(),
            );
            stage_flags += ")";

            writer.add_line("CGenShaderInstance {");
            writer.add_line(format!("key: CGenShaderKey::make(ID, {key_list}),"));
            writer.add_line(format!("stage_flags: {stage_flags},"));
            writer.add_line("},");
        }
    }
    writer.new_line();

    {
        let mut writer = writer.add_block(
            &["pub static SHADER_DEF: CGenShaderDef = CGenShaderDef {"],
            &["};"],
        );
        writer.add_line("id: ID,");
        writer.add_line(format!("name: \"{}\",", shader.name));
        writer.add_line("path: SHADER_PATH.path(),");
        writer.add_line("options: &SHADER_OPTIONS,");
        writer.add_line("instances: &SHADER_INSTANCES,");
    }
    writer.new_line();
    {
        writer.add_line(format!("pub struct {};", shader.name));
        let mut writer = writer.add_block(&[format!("impl {} {{", shader.name)], &["}"]);
        writer.add_line("pub fn def() -> &'static CGenShaderDef { &SHADER_DEF} ");
    }
    writer.new_line();

    writer.build()
}
