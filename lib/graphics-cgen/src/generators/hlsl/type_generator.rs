use heck::ToSnakeCase;

use crate::{
    generators::{
        file_writer::FileWriter, hlsl::utils::get_hlsl_typestring, product::Product, CGenVariant,
        GeneratorContext,
    },
    model::{CGenType, Model, StructMember},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for cgen_type_ref in model.object_iter::<CGenType>() {
        if let Some(content) = match cgen_type_ref.object() {
            CGenType::Native(_) => None,
            CGenType::Struct(_) => Some(generate_hlsl_struct(ctx, cgen_type_ref.object())),
        } {
            products.push(Product::new(
                CGenVariant::Hlsl,
                GeneratorContext::get_object_rel_path(cgen_type_ref.object(), CGenVariant::Hlsl),
                content.into_bytes(),
            ));
        }
    }
    products
}

fn get_member_declaration(model: &Model, member: &StructMember) -> String {
    let typestring = get_hlsl_typestring(member.ty_ref.get(model));

    format!("{} {};", typestring, member.name)
}

fn generate_hlsl_struct<'a>(ctx: &GeneratorContext<'a>, ty: &CGenType) -> String {
    let struct_def = ty.struct_type();
    let mut writer = FileWriter::new();

    // header
    writer.add_line(format!(
        "#ifndef TYPE_{}",
        struct_def.name.to_snake_case().to_uppercase()
    ));
    writer.add_line(format!(
        "#define TYPE_{}",
        struct_def.name.to_snake_case().to_uppercase()
    ));
    writer.new_line();

    writer.indent();

    // dependencies
    let deps = GeneratorContext::get_type_dependencies(ty);
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
    writer.add_line(format!("struct {} {{", struct_def.name));

    writer.indent();
    for m in &struct_def.members {
        writer.add_line(get_member_declaration(ctx.model, m));
    }
    writer.unindent();

    writer.add_line(format!("}}; // {}", struct_def.name));

    writer.new_line();

    writer.unindent();

    // footer
    writer.add_line("#endif");

    // finalize
    writer.build()
}
