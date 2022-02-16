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
            CGenType::BitField(_) => Some(generate_hlsl_bitfield(ctx, ty_ref.object())),
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
        let mut writer = writer.add_block(
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
                    CGenType::Struct(_) | CGenType::BitField(_) => {
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
            let mut writer = writer.add_block(&[format!("struct {} {{", struct_ty.name)], &["};"]);
            for m in &struct_ty.members {
                writer.add_line(member_declaration(ctx.model, m));
            }
        }

        writer.new_line();
    }

    // finalize
    writer.build()
}

fn generate_hlsl_bitfield<'a>(_ctx: &GeneratorContext<'a>, ty: &CGenType) -> String {
    let mut writer = FileWriter::new();
    let bf_type = ty.bitfield_type();

    {
        // write guard scope
        let mut writer = writer.add_block(
            &[
                format!("#ifndef TYPE_{}", bf_type.name.to_shouty_snake_case()),
                format!("#define TYPE_{}", bf_type.name.to_shouty_snake_case()),
            ],
            &["#endif"],
        );
        writer.new_line();

        {
            let bf_name = &bf_type.name;
            let mut writer = writer.add_block(&[format!("struct {bf_name} {{")], &["};"]);
            writer.add_line("uint value;");
            writer.new_line();
            {
                let mut writer =
                    writer.add_block(&[format!("{bf_name} operator|({bf_name} v ) {{")], &["}"]);
                writer.add_line(format!("{bf_name} ret = {{ (value | v.value)}} ;"));
                writer.add_line("return ret;");
            }
            {
                let mut writer = writer.add_block(
                    &[format!("bool is_set({0} flags ) {{", bf_type.name)],
                    &["}"],
                );
                writer.add_line("return (value & flags.value) == flags.value;");
            }
            {
                let mut writer =
                    writer.add_block(&[format!("void set({0} flags) {{", bf_type.name)], &["}"]);
                writer.add_line("value |= flags.value;");
            }
            {
                let mut writer =
                    writer.add_block(&[format!("void unset({0} flags) {{", bf_type.name)], &["}"]);
                writer.add_line("value &= ~flags.value;");
            }
        }
        writer.new_line();

        let mut hex_value = 1;
        for value in &bf_type.values {
            writer.add_line(format!(
                "static const {0} {0}_{1:16} = {{ 0x{2:08x} }};",
                bf_type.name, value, hex_value
            ));
            hex_value <<= 1;
        }

        writer.new_line();
    }

    writer.build()
}
