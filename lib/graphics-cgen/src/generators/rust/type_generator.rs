use heck::ToSnakeCase;

use crate::{
    generators::{
        file_writer::FileWriter, product::Product, rust::utils::get_rust_typestring, CGenVariant,
        GeneratorContext,
    },
    model::{CGenType, Model, StructMember},
    struct_layout::{StructLayout, StructMemberLayout},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for ty_ref in model.object_iter::<CGenType>() {
        if let Some(content) = match ty_ref.object() {
            CGenType::Native(_) => None,
            CGenType::Struct(_) => {
                let ty_layout = ctx.struct_layouts.get(ty_ref.id());
                if ty_layout.is_some() {
                    Some(generate_rust_struct(
                        ctx,
                        ty_ref.id(),
                        ty_ref.object(),
                        ty_layout.unwrap(),
                    ))
                } else {
                    None
                }
            }
        } {
            products.push(Product::new(
                CGenVariant::Rust,
                GeneratorContext::get_object_rel_path(ty_ref.object(), CGenVariant::Rust),
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

fn get_member_declaration(
    model: &Model,
    member: &StructMember,
    layout: &StructMemberLayout,
) -> String {
    let typestring = get_rust_typestring(member.ty_handle.get(model));

    if let Some(array_len) = member.array_len {
        format!("pub {}: [{}; {}],", member.name, typestring, array_len)
    } else {
        format!("pub {}: {},", member.name, typestring)
    }
}

fn generate_rust_struct(
    ctx: &GeneratorContext<'_>,
    ty_id: u32,
    ty: &CGenType,
    ty_layout: &StructLayout,
) -> String {
    let struct_ty = ty.struct_type();
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
    let deps = ty.get_type_dependencies();

    if !deps.is_empty() {
        let mut has_native_types = false;
        for ty_handle in &deps {
            let ty = ty_handle.get(ctx.model);
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

    // write layout (debug purpose)
    {
        writer.add_line("/*");
        writer.add_line(format!("{:#?}", ty_layout));
        writer.add_line("*/");
    }

    // write type def
    {
        writer.add_line("static TYPE_DEF: CGenTypeDef = CGenTypeDef{ ");
        writer.indent();
        writer.add_line(format!("name: \"{}\",", struct_ty.name));
        writer.add_line(format!("id: {},", ty_id));
        writer.add_line(format!("size: {},", ty_layout.padded_size));
        writer.unindent();
        writer.add_line("}; ");
        writer.new_line();
        writer.add_line(format!(
            "static_assertions::const_assert_eq!(mem::size_of::<{}>(), {});",
            struct_ty.name, ty_layout.padded_size
        ));
        writer.new_line();
    }

    // struct
    writer.add_line("#[derive(Default, Clone, Copy)]");
    writer.add_line("#[repr(C)]");
    writer.add_line(format!("pub struct {} {{", struct_ty.name));
    writer.indent();
    let member_len = struct_ty.members.len();
    let mut cur_offset = 0;
    let mut padding_index = 0;
    for i in 0..member_len {
        let struct_m = &struct_ty.members[i];
        let layout_m = &ty_layout.members[i];
        if layout_m.offset != cur_offset {
            let padding_size = layout_m.offset - cur_offset;

            writer.add_line(format!("pad_{}: [u8;{}],", padding_index, padding_size));

            padding_index += 1;
            cur_offset += padding_size;
        }
        writer.add_line(get_member_declaration(ctx.model, struct_m, layout_m));
        cur_offset += layout_m.padded_size;
    }
    if ty_layout.padded_size != cur_offset {
        let padding_size = ty_layout.padded_size - cur_offset;
        writer.add_line(format!("pad_{}: [u8;{}],", padding_index, padding_size));
    }
    writer.unindent();
    writer.add_line("}");
    writer.new_line();

    // impl
    {
        writer.add_line(format!("impl {} {{", struct_ty.name));
        writer.indent();

        // impl: id
        writer.add_line(format!("pub const fn id() -> u32 {{ {}  }}", ty_id));
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
