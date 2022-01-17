use heck::ToSnakeCase;

use crate::{
    db::CGenType,
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
};

use super::utils::get_member_declaration;

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
                GeneratorContext::get_object_rel_path(ty_ref.object(), CGenVariant::Hlsl),
                content.into_bytes(),
            ));
        }
    }
    products
}

fn generate_hlsl_struct<'a>(ctx: &GeneratorContext<'a>, ty: &CGenType) -> String {
    let struct_ty = ty.struct_type();
    let mut writer = FileWriter::new();

    // header
    writer.add_line(format!(
        "#ifndef TYPE_{}",
        struct_ty.name.to_snake_case().to_uppercase()
    ));
    writer.add_line(format!(
        "#define TYPE_{}",
        struct_ty.name.to_snake_case().to_uppercase()
    ));
    writer.new_line();

    writer.indent();

    // dependencies
    let deps = ty.get_type_dependencies();
    if !deps.is_empty() {
        for ty_ref in deps {
            let ty = ty_ref.get(ctx.model);
            match ty {
                CGenType::Native(_) => (),
                CGenType::Struct(_) => {
                    let dep_filename = GeneratorContext::get_object_filename(ty, CGenVariant::Hlsl);
                    writer.add_line(format!("#include \"{}\"", dep_filename.as_str()));
                }
            }
        }
        writer.new_line();
    }

    // struct
    writer.add_line(format!("struct {} {{", struct_ty.name));

    writer.indent();
    for m in &struct_ty.members {
        writer.add_line(get_member_declaration(ctx.model, m));
    }
    writer.unindent();

    writer.add_line("};");

    writer.new_line();

    writer.unindent();

    // footer
    writer.add_line("#endif");

    // finalize
    writer.build()
}
