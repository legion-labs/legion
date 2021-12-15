use heck::SnakeCase;

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
    for cgen_type in model.object_iter::<CGenType>() {
        match cgen_type {
            CGenType::Native(_) => None,
            CGenType::Struct(_) => Some(generate_rust_struct(&ctx, cgen_type)),
        }
        .map(|content| {
            products.push(Product::new(
                CGenVariant::Rust,
                GeneratorContext::get_object_rel_path(cgen_type, CGenVariant::Rust),
                content.into_bytes(),
            ))
        });
    }

    if !products.is_empty() {
        let mut mod_path = GeneratorContext::get_object_folder::<CGenType>();
        mod_path.push("mod.rs");

        let mut writer = FileWriter::new();
        for product in &products {
            let filename = product.path().file_stem().unwrap();   
            writer.add_line(format!("pub(crate) mod {};", &filename));
            writer.add_line("#[allow(unused_imports)]");
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

fn get_member_declaration(model: &Model, member: &StructMember) -> String {
    let typestring = get_rust_typestring(model, member.object_id);

    format!("pub(crate) {}: {},", member.name, typestring)
}

fn generate_rust_struct<'a>(ctx: &GeneratorContext<'a>, ty: &CGenType) -> String {
    let struct_def = ty.struct_type();
    let mut writer = FileWriter::new();

    // dependencies
    let deps = GeneratorContext::get_type_dependencies(ty);

    if !deps.is_empty() {
        let mut has_native_types = false;
        for object_id in &deps {
            let dep_ty = ctx.model.get_from_objectid::<CGenType>(*object_id).unwrap();
            match dep_ty {
                CGenType::Native(_) => {
                    has_native_types = true;
                }
                CGenType::Struct(e) => {
                    writer.add_line(format!(
                        "use super::{}::{};",
                        e.name.to_snake_case(),
                        e.name
                    ));
                }
            }
        }
        if has_native_types {
            writer.add_line(format!("use lgn_graphics_cgen_runtime::prelude::*;"));
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
