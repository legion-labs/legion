use heck::{ToShoutySnakeCase, ToSnakeCase};

use crate::{
    db::{CGenType, DescriptorSet, Model, ModelObject, PipelineLayout, Shader},
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let content = generate(ctx);
    products.push(Product::new(
        CGenVariant::Rust,
        "mod.rs".to_string(),
        content.into_bytes(),
    ));

    products
}

fn generate(ctx: &GeneratorContext<'_>) -> String {
    let mut writer = FileWriter::new();

    // write dependencies
    let model = ctx.model;
    writer.add_line("use lgn_graphics_api::DeviceContext;");
    writer.add_line("use lgn_graphics_cgen_runtime::{CGenCrateID,CGenRegistry};");
    writer.new_line();

    // crate id
    {
        writer.add_line(format!(
            "pub const CRATE_ID : CGenCrateID = CGenCrateID({:#X});",
            ctx.crate_id
        ));
        writer.new_line();
    }

    // fn initialize
    {
        let mut writer = writer.add_block(
            &["pub fn initialize(device_context: &DeviceContext) -> CGenRegistry   {"],
            &["}"],
        );
        writer.add_line("let mut registry = CGenRegistry::new( CRATE_ID, shutdown  );");
        writer.new_line();

        for ty_ref in model.object_iter::<CGenType>() {
            match ty_ref.object() {
                CGenType::Native(_) => {
                    writer.add_line(format!(
                        "registry.add_type(lgn_graphics_cgen_runtime::{}::def());",
                        ty_ref.object().name()
                    ));
                }
                CGenType::Struct(_) | CGenType::BitField(_) => {
                    writer.add_line(format!(
                        "registry.add_type(cgen_type::{}::def());",
                        ty_ref.object().name()
                    ));
                }
            }
        }
        writer.new_line();

        for descriptor_set_ref in model.object_iter::<DescriptorSet>() {
            writer.add_line(format!(
                "registry.add_descriptor_set(device_context, descriptor_set::{}::def());",
                descriptor_set_ref.object().name
            ));
        }
        writer.new_line();

        for pipeline_layout_ref in model.object_iter::<PipelineLayout>() {
            writer.add_line(format!(
                "registry.add_pipeline_layout(device_context, pipeline_layout::{}::def());",
                pipeline_layout_ref.object().name
            ));
        }
        writer.new_line();

        for shader_ref in model.object_iter::<Shader>() {
            writer.add_line(format!(
                "registry.add_shader_def(shader::{}::def());",
                shader_ref.object().name
            ));
        }
        writer.new_line();

        for descriptor_set_ref in model.object_iter::<DescriptorSet>() {
            writer.add_line(format!(
                "descriptor_set::{}::initialize(registry.descriptor_set_layout({}));",
                descriptor_set_ref.object().name,
                descriptor_set_ref.id()
            ));
        }
        writer.new_line();

        for pipeline_layout_ref in model.object_iter::<PipelineLayout>() {
            writer.add_line(format!(
                "pipeline_layout::{}::initialize(registry.pipeline_layout({}));",
                pipeline_layout_ref.object().name,
                pipeline_layout_ref.id()
            ));
        }
        writer.new_line();

        writer.add_line("shader_files::force_export();");
        writer.new_line();

        writer.add_line("registry");
    }

    writer.new_line();

    // fn shutdown
    {
        let mut writer = writer.add_block(&["pub fn shutdown() {"], &["}"]);

        for descriptor_set_ref in model.object_iter::<DescriptorSet>() {
            writer.add_line(format!(
                "descriptor_set::{}::shutdown();",
                descriptor_set_ref.object().name
            ));
        }
        writer.new_line();

        for pipeline_layout_ref in model.object_iter::<PipelineLayout>() {
            writer.add_line(format!(
                "pipeline_layout::{}::shutdown();",
                pipeline_layout_ref.object().name
            ));
        }
    }

    writer.new_line();

    // add shader files
    {
        let infos: Vec<_> = ctx
            .model
            .object_iter::<CGenType>()
            .filter_map(|cgen_type| match cgen_type.object() {
                CGenType::Native(_) => None,
                CGenType::BitField(_) | CGenType::Struct(_) => {
                    Some(embedded_fs_info(ctx, cgen_type.object()))
                }
            })
            .chain(
                ctx.model
                    .object_iter::<PipelineLayout>()
                    .map(|pipeline_layout| embedded_fs_info(ctx, pipeline_layout.object())),
            )
            .chain(
                ctx.model
                    .object_iter::<DescriptorSet>()
                    .map(|descriptor_set| embedded_fs_info(ctx, descriptor_set.object())),
            )
            .collect();
        let mut writer = writer.add_block(&["#[rustfmt::skip]", "mod shader_files {"], &["}"]);

        writer.add_line("pub(super) fn force_export() {} ");
        writer.new_line();

        for (var_name, rel_path, crate_path) in infos {
            let mut writer = writer.add_block(
                &[
                    "#[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]".to_string(),
                    format!("static {}: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(", var_name),
                ],
                &[");"],
            );
            writer.add_line(format!("\"{}\",", crate_path));
            writer.add_line(format!(
                "include_bytes!(concat!(env!(\"OUT_DIR\"), \"/hlsl/{}\")),",
                rel_path
            ));
            writer.add_line("None".to_string());
        }
    }

    writer.new_line();

    write_mod::<CGenType>(model, &mut writer);
    write_mod::<DescriptorSet>(model, &mut writer);
    write_mod::<PipelineLayout>(model, &mut writer);
    write_mod::<Shader>(model, &mut writer);

    writer.build()
}

trait SkipInclude: ModelObject {
    fn skip_include(&self) -> bool {
        false
    }
}

impl SkipInclude for CGenType {
    fn skip_include(&self) -> bool {
        matches!(self, CGenType::Native(_))
    }
}
impl SkipInclude for DescriptorSet {}
impl SkipInclude for PipelineLayout {}
impl SkipInclude for Shader {}

fn write_mod<T>(model: &Model, writer: &mut FileWriter)
where
    T: ModelObject + SkipInclude,
{
    if model.size::<T>() > 0 {
        let folder = GeneratorContext::object_folder::<T>();
        let mut writer = writer.add_block(
            &[
                "#[allow(dead_code, clippy::needless_range_loop, clippy::derivable_impls)]"
                    .to_string(),
                format!("pub mod {} {{", folder),
            ],
            &["}"],
        );

        for obj_ref in model.object_iter::<T>() {
            let mod_name = obj_ref.object().name().to_snake_case();
            if obj_ref.object().skip_include() {
                continue;
            }
            {
                let mut writer = writer.add_block(&[format!("pub mod {} {{", mod_name)], &["}"]);
                writer.add_line(format!(
                    "include!(concat!(env!(\"OUT_DIR\"), \"/{}/{}\"));",
                    CGenVariant::Rust.dir(),
                    GeneratorContext::object_relative_path(obj_ref.object(), CGenVariant::Rust)
                ));
            }
            writer.add_lines(&[
                "#[allow(unused_imports)]".to_string(),
                format!("pub use self::{}::*;", mod_name),
            ]);
        }
    }
    writer.new_line();
}

fn embedded_fs_info(
    ctx: &GeneratorContext<'_>,
    obj: &impl ModelObject,
) -> (String, String, String) {
    (
        obj.name().to_shouty_snake_case(),
        GeneratorContext::object_relative_path(obj, CGenVariant::Hlsl),
        ctx.embedded_fs_path(obj, CGenVariant::Hlsl),
    )
}
