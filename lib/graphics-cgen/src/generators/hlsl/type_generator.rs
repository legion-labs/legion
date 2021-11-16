use crate::{generators::{CGenVariant, Generator, GeneratorContext, Product, file_writer::FileWriter, hlsl::utils::get_hlsl_typestring}, model::{CGenType, Model, StructMember}};

#[derive(Default)]
pub struct TypeGenerator {}

impl Generator for TypeGenerator {
    fn run(&self, ctx: &GeneratorContext<'_>) -> Vec<Product> {
        let mut products = Vec::new();
        let model = ctx.model;
        let cgen_types = model.object_iter::<CGenType>().unwrap_or_default();
        for cgen_type in cgen_types {
            dbg!(&cgen_type);
            match cgen_type {
                CGenType::Native(_) => None,
                CGenType::Struct(_) => Some(generate_hlsl_struct(&ctx, cgen_type)),
            }
            .map(|content| {
                products.push(Product {
                    path: ctx.get_type_abspath(cgen_type, CGenVariant::Hlsl),
                    content,
                })
            });
        }
        dbg!(&products);
        products
    }
}

fn get_member_declaration(model: &Model, member: &StructMember) -> String {
    let typestring = get_hlsl_typestring(model, member.type_key);

    format!("{} {};", typestring, member.name)
}

fn generate_hlsl_struct<'a>(ctx: &GeneratorContext<'a>, ty: &CGenType) -> String {
    let struct_def = ty.struct_type();
    let mut writer = FileWriter::new();

    // header
    writer.add_line(format!("#ifndef TYPE_{}", struct_def.name.to_uppercase()));
    writer.add_line(format!("#define TYPE_{}", struct_def.name.to_uppercase()));
    writer.new_line();

    writer.indent();

    // dependencies
    let deps = ctx.get_type_dependencies(ty).unwrap();

    if !deps.is_empty() {
        for key in &deps {
            let dep_ty = ctx.model.get::<CGenType>(*key).unwrap();
            let dep_path = ctx.get_type_abspath(dep_ty, CGenVariant::Hlsl);            
            writer.add_line(format!(
                "#include \"{}\"",
                dep_path.as_os_str().to_str().unwrap()
            ));
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
    writer.add_line("#endif".to_string());

    // finalize
    writer.to_string()
}
