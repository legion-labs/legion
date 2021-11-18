use heck::SnakeCase;
use relative_path::RelativePath;

use crate::{
    generators::{
        file_writer::FileWriter, product::Product, rust::utils::get_rust_typestring, CGenVariant,
        GeneratorContext,
    },
    model::{CGenType, Model, StructMember},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    let cgen_types = model.object_iter::<CGenType>().unwrap_or_default();
    for cgen_type in cgen_types {
        match cgen_type {
            CGenType::Native(_) => None,
            CGenType::Struct(_) => Some(generate_rust_struct(&ctx, cgen_type)),
        }
        .map(|content| {
            products.push(Product::new(
                CGenVariant::Rust,
                ctx.get_rel_type_path(cgen_type, CGenVariant::Rust),
                content,
            ))
        });
    }

    if !products.is_empty() {
        let mut mod_path = RelativePath::new("types/mod.rs");
        let mut writer = FileWriter::new();
        for product in &products {
            let filename = product.path().file_stem().unwrap();
            writer.add_line(format!("pub(crate) mod {};", &filename));
        }
        products.push(Product::new(
            CGenVariant::Rust,
            mod_path.to_relative_path_buf(),
            writer.to_string(),
        ));
    }

    products
}

fn get_member_declaration(model: &Model, member: &StructMember) -> String {
    let typestring = get_rust_typestring(model, member.type_key);

    format!("pub(crate) {}: {},", member.name, typestring)
}

fn generate_rust_struct<'a>(ctx: &GeneratorContext<'a>, ty: &CGenType) -> String {
    let struct_def = ty.struct_type();
    let mut writer = FileWriter::new();

    // dependencies
    let deps = ctx.get_type_dependencies(ty).unwrap();

    if !deps.is_empty() {
        for key in &deps {
            let dep_ty = ctx.model.get::<CGenType>(*key).unwrap();
            match dep_ty {
                CGenType::Native(e) => {}
                CGenType::Struct(e) => {
                    writer.add_line(format!("use super::{}::{};", e.name.to_snake_case(), e.name));
                }
            }
        }
        writer.new_line();
    }

    // struct
    writer.add_line(format!("pub struct {} {{", struct_def.name));

    writer.indent();
    for m in &struct_def.members {
        writer.add_line(get_member_declaration(ctx.model, m));
    }
    writer.unindent();

    writer.add_line(format!("}} // {}", struct_def.name));

    writer.new_line();

    // finalize
    writer.to_string()
}
