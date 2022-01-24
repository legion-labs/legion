use heck::ToShoutySnakeCase;

use crate::{
    db::CGenType,
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
};

use super::utils::member_declaration;

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for ty_ref in model.object_iter::<CGenType>() {
        if let Some(content) = match ty_ref.object() {
            CGenType::Native(_) => None,
            CGenType::Struct(_) => {
                let ty_layout = ctx.struct_layouts.get(ty_ref.id());
                if ty_layout.is_some() {
                    Some(generate_hlsl_struct(ctx, ty_ref.object()))
                } else {
                    None
                }
            }
        } {
            products.push(Product::new(
                CGenVariant::Hlsl,
                GeneratorContext::object_relative_path(ty_ref.object(), CGenVariant::Hlsl),
                content.into_bytes(),
            ));
        }
    }
    products
}

fn generate_hlsl_struct<'a>(ctx: &GeneratorContext<'a>, ty: &CGenType) -> String {
    let struct_ty = ty.struct_type();
    let mut writer = FileWriter::new();

    {
        // write guard scope
        let mut writer = writer.new_block(
            &[
                format!("#ifndef TYPE_{}", struct_ty.name.to_shouty_snake_case()),
                format!("#define TYPE_{}", struct_ty.name.to_shouty_snake_case()),
            ],
            &["#endif"],
        );
        // dependencies
        let deps = ty.get_type_dependencies();
        if !deps.is_empty() {
            for ty_ref in deps {
                let ty = ty_ref.get(ctx.model);
                match ty {
                    CGenType::Native(_) => (),
                    CGenType::Struct(_) => {
                        writer.add_line(format!(
                            "#include \"{}\"",
                            ctx.embedded_fs_path(ty, CGenVariant::Hlsl)
                        ));
                    }
                }
            }
            writer.new_line();
        }

        // struct
        {
            let mut writer = writer.new_block(&[format!("struct {} {{", struct_ty.name)], &["};"]);
            for m in &struct_ty.members {
                writer.add_line(member_declaration(ctx.model, m));
            }
        }

        writer.new_line();
    }

    // finalize
    writer.build()
}
