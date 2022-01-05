use heck::ToSnakeCase;

use crate::{
    generators::{
        file_writer::FileWriter, product::Product, rust::utils::get_rust_typestring, CGenVariant,
        GeneratorContext,
    },
    model::{CGenType, Model, StructMember, CGenTypeRef},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for ty_ref in model.ref_iter::<CGenType>() {
        let ty = ty_ref.get(model);
        if let Some(content) = match ty {
            CGenType::Native(_) => None,
            CGenType::Struct(_) => Some(generate_rust_struct(ctx, ty_ref)),
        } {
            products.push(Product::new(
                CGenVariant::Rust,
                GeneratorContext::get_object_rel_path(ty, CGenVariant::Rust),
                content.into_bytes(),
            ));
        }
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
            writer.build().into_bytes(),
        ));
    }

    products
}

fn get_member_declaration(model: &Model, member: &StructMember) -> String {
    let typestring = get_rust_typestring(member.ty_ref.get(model));

    format!("pub {}: {},", member.name, typestring)
}

fn generate_rust_struct<'a>(ctx: &GeneratorContext<'a>, ty_ref: CGenTypeRef) -> String {
    let ty = ty_ref.get(ctx.model);
    let struct_def = ty.struct_type();
    let mut writer = FileWriter::new();

    // global dependencies
    writer.add_line("use std::mem;");
    writer.new_line();

    writer.add_line("use lgn_graphics_cgen_runtime::{");
    writer.indent();
    writer.add_line("CGenTypeDef,");    
    writer.unindent();
    writer.add_line("};");
    writer.new_line();

    // local dependencies
    let deps = GeneratorContext::get_type_dependencies(ty);

    if !deps.is_empty() {
        let mut has_native_types = false;
        for ty_ref in &deps {
            let ty = ty_ref.get(ctx.model);
            match ty {
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
            writer.add_line("use lgn_graphics_cgen_runtime::prelude::*;".to_string());
        }
        writer.new_line();
    }

    // write type def
    {
        writer.add_line(
            "static TYPE_DEF: CGenTypeDef = CGenTypeDef{ ",
        );
        writer.indent();
        writer.add_line(format!("name: \"{}\",", struct_def.name));
        writer.add_line(format!("id: {},", ty_ref.id()));
        writer.add_line(format!("size: mem::size_of::<{}>(),", struct_def.name));
        writer.unindent();
        writer.add_line("}; ");
        writer.new_line();
    }

    // struct
    writer.add_line("#[derive(Default, Clone, Copy)]");
    writer.add_line("#[repr(C)]");
    writer.add_line(format!("pub struct {} {{", struct_def.name));
    writer.indent();
    for m in &struct_def.members {
        writer.add_line(get_member_declaration(ctx.model, m));
    }
    writer.unindent();
    writer.add_line("}");
    writer.new_line();

    // impl
    {
        writer.add_line(format!("impl {} {{", struct_def.name));
        writer.indent();        

        // impl: id
        writer.add_line(format!(
            "pub const fn id() -> u32 {{ {}  }}",
            ty_ref.id()
        ));
        writer.new_line();

        // impl: def
        writer.add_line("pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }");
        writer.new_line();

        
        writer.unindent();
        writer.add_line("}");
        writer.new_line();
    }

    // finalize
    writer.build()
}
