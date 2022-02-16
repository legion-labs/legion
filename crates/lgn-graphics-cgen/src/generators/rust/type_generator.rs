use heck::ToSnakeCase;

use crate::{
    db::CGenType,
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
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
            CGenType::BitField(_) => {
                Some(generate_rust_bitfield(ctx, ty_ref.id(), ty_ref.object()))
            }
        } {
            products.push(Product::new(
                CGenVariant::Rust,
                GeneratorContext::object_relative_path(ty_ref.object(), CGenVariant::Rust),
                content.into_bytes(),
            ));
        }
    }

    products
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

    {
        let mut writer = writer.add_block(&["use lgn_graphics_cgen_runtime::{"], &["};"]);
        writer.add_line("CGenTypeDef,");
    }
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
                CGenType::BitField(e) => {
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
        let mut writer =
            writer.add_block(&["static TYPE_DEF: CGenTypeDef = CGenTypeDef{"], &["};"]);
        writer.add_lines(&[
            format!("name: \"{}\",", struct_ty.name),
            format!("size: {},", ty_layout.padded_size),
        ]);
    }

    writer.new_line();
    writer.add_line(format!(
        "static_assertions::const_assert_eq!(mem::size_of::<{}>(), {});",
        struct_ty.name, ty_layout.padded_size
    ));
    writer.new_line();

    // struct
    {
        let mut writer = writer.add_block(
            &[
                "#[derive(Clone, Copy)]".to_string(),
                "#[repr(C)]".to_string(),
                format!("pub struct {} {{", struct_ty.name),
            ],
            &["}"],
        );
        writer.add_line(format!("data: [u8;{}]", ty_layout.padded_size));
    }

    writer.new_line();

    // impl
    {
        let mut writer = writer.add_block(
            &[
                "#[allow(clippy::trivially_copy_pass_by_ref)]".to_string(),
                format!("impl {} {{", struct_ty.name),
            ],
            &["}"],
        );
        // impl: id
        writer.add_line(format!("pub const fn id() -> u32 {{ {}  }}", ty_id));
        writer.new_line();

        // impl: def
        writer.add_line("pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }");
        writer.new_line();

        // members
        let member_len = struct_ty.members.len();
        for i in 0..member_len {
            let struct_member = &struct_ty.members[i];
            let struct_member_layout = &ty_layout.members[i];
            let struct_member_type = struct_member.ty_handle.get(ctx.model);
            let struct_member_type_name = struct_member_type.to_rust_name();

            writer.add_line("//");
            writer.add_line(format!("// member : {}", struct_member.name));
            writer.add_line(format!("// offset : {}", struct_member_layout.offset));
            writer.add_line(format!("// size : {}", struct_member_layout.padded_size));
            writer.add_line("//");

            if let Some(array_len) = struct_member.array_len {
                {
                    let mut writer = writer.add_block(
                        &[format!(
                            "pub fn set_{}(&mut self, values: [{};{}]) {{ ",
                            struct_member.name, struct_member_type_name, array_len
                        )],
                        &["}"],
                    );

                    {
                        let mut writer =
                            writer.add_block(&[format!("for i in 0..{} {{", array_len)], &["}"]);
                        writer.add_line(format!(
                            "self.set_{}_element(i, values[i]);",
                            struct_member.name,
                        ));
                    }
                }

                writer.new_line();

                // set element by index
                {
                    let mut writer = writer.add_block(
                        &[format!(
                            "pub fn set_{}_element(&mut self, index: usize, value: {}) {{ ",
                            struct_member.name, struct_member_type_name
                        )],
                        &["}"],
                    );
                    writer.add_line(format!("assert!(index<{});", array_len));
                    writer.add_line(format!(
                        "self.set::<{}>({} + index * {} , value);",
                        struct_member_type_name,
                        struct_member_layout.offset,
                        struct_member_layout.array_stride
                    ));
                }

                writer.new_line();

                // get all elements
                {
                    let mut writer = writer.add_block(
                        &[format!(
                            "pub fn {}(&self) ->  [{};{}] {{ ",
                            struct_member.name, struct_member_type_name, array_len
                        )],
                        &["}"],
                    );
                    writer.add_line(format!("self.get({})", struct_member_layout.offset,));
                }

                writer.new_line();

                // get element by index
                {
                    let mut writer = writer.add_block(
                        &[format!(
                            "pub fn {}_element(&self, index: usize) -> {} {{ ",
                            struct_member.name, struct_member_type_name
                        )],
                        &["}"],
                    );
                    writer.add_line(format!("assert!(index<{});", array_len));
                    writer.add_line(format!(
                        "self.get::<{}>({} + index * {})",
                        struct_member_type_name,
                        struct_member_layout.offset,
                        struct_member_layout.array_stride
                    ));
                }
            } else {
                // set
                {
                    let mut writer = writer.add_block(
                        &[format!(
                            "pub fn set_{}(&mut self, value: {}) {{ ",
                            struct_member.name, struct_member_type_name
                        )],
                        &["}"],
                    );
                    writer.add_line(format!("self.set({}, value);", struct_member_layout.offset,));
                }

                writer.new_line();

                // get
                {
                    let mut writer = writer.add_block(
                        &[format!(
                            "pub fn {}(&self) -> {} {{ ",
                            struct_member.name, struct_member_type_name
                        )],
                        &["}"],
                    );
                    writer.add_line(format!("self.get({})", struct_member_layout.offset,));
                }
            }
            writer.new_line();
        }

        {
            let mut writer = writer.add_block(
                &[
                    "#[allow(unsafe_code)]",
                    "fn set<T: Copy>(&mut self, offset: usize, value: T) {",
                ],
                &["}"],
            );
            {
                let mut writer = writer.add_block(&["unsafe {"], &["}"]);
                writer.add_lines(&[
                    "let p = self.data.as_mut_ptr();",
                    "let p = p.add(offset as usize);",
                    "let p = p.cast::<T>();",
                    "p.write(value);",
                ]);
            }
        }
        writer.new_line();

        {
            let mut writer = writer.add_block(
                &[
                    "#[allow(unsafe_code)]",
                    "fn get<T: Copy>(&self, offset: usize) -> T {",
                ],
                &["}"],
            );
            {
                let mut writer = writer.add_block(&["unsafe {"], &["}"]);
                writer.add_lines(&[
                    "let p = self.data.as_ptr();",
                    "let p = p.add(offset as usize);",
                    "let p = p.cast::<T>();",
                    "*p",
                ]);
            }
        }
    }

    writer.new_line();

    // impl Default
    {
        let mut writer =
            writer.add_block(&[format!("impl Default for {} {{", struct_ty.name)], &["}"]);

        {
            let mut writer = writer.add_block(&["fn default() -> Self {"], &["}"]);
            writer.add_line("let mut ret = Self {");
            writer.add_line(format!("data: [0;{}]", ty_layout.padded_size));
            writer.add_line("};");

            let member_len = struct_ty.members.len();
            for i in 0..member_len {
                let struct_member = &struct_ty.members[i];
                let struct_member_type = struct_member.ty_handle.get(ctx.model);
                let struct_member_type_name = struct_member_type.to_rust_name();

                if let Some(array_len) = struct_member.array_len {
                    writer.add_line(format!(
                        "ret.set_{}([{}::default();{}]);",
                        struct_member.name, struct_member_type_name, array_len
                    ));
                } else {
                    writer.add_line(format!(
                        "ret.set_{}({}::default());",
                        struct_member.name, struct_member_type_name
                    ));
                }
            }

            writer.add_line("ret");
        }
    }

    writer.new_line();

    // finalize
    writer.build()
}

fn generate_rust_bitfield(_ctx: &GeneratorContext<'_>, _ty_id: u32, ty: &CGenType) -> String {
    let mut writer = FileWriter::new();
    let bf_type = ty.bitfield_type();

    {
        let mut writer = writer.add_block(&["use lgn_graphics_cgen_runtime::{"], &["};"]);
        writer.add_line("CGenTypeDef,");
    }
    writer.new_line();
    {
        let mut writer =
            writer.add_block(&["static TYPE_DEF: CGenTypeDef = CGenTypeDef{"], &["};"]);
        writer.add_lines(&[
            format!("name: \"{}\",", bf_type.name),
            format!("size: std::mem::size_of::<{}>(),", bf_type.name),
        ]);
    }
    writer.new_line();
    {
        let mut writer = writer.add_block(&["bitflags::bitflags!{"], &["}"]);
        {
            let mut writer =
                writer.add_block(&[format!("pub struct {} : u32 {{", bf_type.name)], &["}"]);

            let mut hex_value = 1;
            for value in &bf_type.values {
                writer.add_line(format!("const {:16} = 0x{:08x};", value, hex_value));
                hex_value <<= 1;
            }
        }
    }
    writer.new_line();

    {
        let mut writer = writer.add_block(&[format!("impl {} {{", bf_type.name)], &["}"]);
        writer.add_line("pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }");
    }

    writer.new_line();

    writer.build()
}
