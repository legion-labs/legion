use heck::ToSnakeCase;

use crate::{
    db::CGenType,
    generators::{
        file_writer::FileWriter, product::Product, rust::utils::get_rust_typestring, CGenVariant,
        GeneratorContext,
    },
    struct_layout::StructLayout,
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for ty_ref in model.object_iter::<CGenType>() {
        if let Some(content) = match ty_ref.object() {
            CGenType::Native(_) => None,
            CGenType::Struct(_) => {
                let ty_layout = ctx.struct_layouts.get(ty_ref.id());
                ty_layout.map(|ty_layout| {
                    generate_rust_struct(ctx, ty_ref.id(), ty_ref.object(), ty_layout)
                })
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

#[allow(clippy::too_many_lines)]
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
    writer.add_line("#[derive(Clone, Copy)]");
    writer.add_line("#[repr(C)]");
    writer.add_line(format!("pub struct {} {{", struct_ty.name));
    writer.indent();
    writer.add_line(format!("data: [u8;{}]", ty_layout.padded_size));
    writer.unindent();
    writer.add_line("}");
    writer.new_line();

    // impl
    {
        writer.add_line("#[allow(clippy::trivially_copy_pass_by_ref)]");
        writer.add_line(format!("impl {} {{", struct_ty.name));
        writer.indent();

        // impl: id
        writer.add_line(format!("pub const fn id() -> u32 {{ {}  }}", ty_id));
        writer.new_line();

        // impl: def
        writer.add_line("pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }");
        writer.new_line();

        // members
        let member_len = struct_ty.members.len();
        for i in 0..member_len {
            let struct_m = &struct_ty.members[i];
            let layout_m = &ty_layout.members[i];
            let ty_m = struct_m.ty_handle.get(ctx.model);
            let ty_string_m = get_rust_typestring(ty_m);

            writer.add_line("//");
            writer.add_line(format!("// member : {}", struct_m.name));
            writer.add_line(format!("// offset : {}", layout_m.offset));
            writer.add_line(format!("// size : {}", layout_m.padded_size));
            writer.add_line("//");

            if let Some(array_len) = struct_m.array_len {
                // set all elements
                writer.add_line(format!(
                    "pub fn set_{}(&mut self, values: [{};{}]) {{ ",
                    struct_m.name, ty_string_m, array_len
                ));
                writer.indent();
                writer.add_line(format!("for i in 0..{} {{", array_len));
                writer.indent();
                writer.add_line(format!("self.set_{}_element(i, values[i]);", struct_m.name,));
                writer.unindent();
                writer.add_line("}");
                writer.unindent();
                writer.add_line("}");
                writer.new_line();
                // set element by index
                writer.add_line(format!(
                    "pub fn set_{}_element(&mut self, index: usize, value: {}) {{ ",
                    struct_m.name, ty_string_m
                ));
                writer.indent();
                writer.add_line(format!("assert!(index<{});", array_len));
                writer.add_line(format!(
                    "self.set::<{}>({} + index * {} , value);",
                    ty_string_m, layout_m.offset, layout_m.array_stride
                ));
                writer.unindent();
                writer.add_line("}");
                writer.new_line();
                // get all elements
                writer.add_line(format!(
                    "pub fn {}(&self) ->  [{};{}] {{ ",
                    struct_m.name, ty_string_m, array_len
                ));
                writer.indent();
                writer.add_line(format!("self.get({})", layout_m.offset,));
                writer.unindent();
                writer.add_line("}");
                writer.new_line();
                // get element by index
                writer.add_line(format!(
                    "pub fn {}_element(&self, index: usize) -> {} {{ ",
                    struct_m.name, ty_string_m
                ));
                writer.indent();
                writer.add_line(format!("assert!(index<{});", array_len));
                writer.add_line(format!(
                    "self.get::<{}>({} + index * {})",
                    ty_string_m, layout_m.offset, layout_m.array_stride
                ));
                writer.unindent();
                writer.add_line("}");
                writer.new_line();
            } else {
                // set
                writer.add_line(format!(
                    "pub fn set_{}(&mut self, value: {}) {{ ",
                    struct_m.name, ty_string_m
                ));
                writer.indent();
                writer.add_line(format!("self.set({}, value);", layout_m.offset,));
                writer.unindent();
                writer.add_line("}");
                writer.new_line();
                // get
                writer.add_line(format!(
                    "pub fn {}(&self) -> {} {{ ",
                    struct_m.name, ty_string_m
                ));
                writer.indent();
                writer.add_line(format!("self.get({})", layout_m.offset,));
                writer.unindent();
                writer.add_line("}");
                writer.new_line();
            }
        }

        writer.add_line("#[allow(unsafe_code)]");
        writer.add_line("fn set<T: Copy>(&mut self, offset: usize, value: T) {");
        writer.indent();
        writer.add_line("unsafe{");
        writer.indent();
        writer.add_line("let p = self.data.as_mut_ptr();");
        writer.add_line("let p = p.add(offset as usize);");
        writer.add_line("let p = p.cast::<T>();");
        writer.add_line("p.write(value);");
        writer.unindent();
        writer.add_line("}");
        writer.unindent();
        writer.add_line("}");
        writer.new_line();

        writer.add_line("#[allow(unsafe_code)]");
        writer.add_line("fn get<T: Copy>(&self, offset: usize) -> T {");
        writer.indent();
        writer.add_line("unsafe{");
        writer.indent();
        writer.add_line("let p = self.data.as_ptr();");
        writer.add_line("let p = p.add(offset as usize);");
        writer.add_line("let p = p.cast::<T>();");
        writer.add_line("*p");
        writer.unindent();
        writer.add_line("}");
        writer.unindent();
        writer.add_line("}");

        writer.unindent();
        writer.add_line("}");
        writer.new_line();
    }

    // impl Default
    {
        writer.add_line(format!("impl Default for {} {{", struct_ty.name));
        writer.indent();

        writer.add_line("fn default() -> Self {");
        writer.indent();
        writer.add_line("let mut ret = Self {");
        writer.add_line(format!("data: [0;{}]", ty_layout.padded_size));
        writer.add_line("};");

        let member_len = struct_ty.members.len();
        for i in 0..member_len {
            let struct_m = &struct_ty.members[i];
            let ty_m = struct_m.ty_handle.get(ctx.model);
            let ty_string_m = get_rust_typestring(ty_m);

            if let Some(array_len) = struct_m.array_len {
                writer.add_line(format!(
                    "ret.set_{}([{}::default();{}]);",
                    struct_m.name, ty_string_m, array_len
                ));
            } else {
                writer.add_line(format!(
                    "ret.set_{}({}::default());",
                    struct_m.name, ty_string_m
                ));
            }
        }

        writer.add_line("ret");

        writer.unindent();
        writer.add_line("}");

        writer.unindent();
        writer.add_line("}");
        writer.new_line();
    }

    // finalize
    writer.build()
}
