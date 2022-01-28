use crate::{
    db::PipelineLayout,
    generators::{file_writer::FileWriter, product::Product, GeneratorContext},
    run::CGenVariant,
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for pipeline_layout_ref in model.object_iter::<PipelineLayout>() {
        let content = generate_hlsl_pipeline_layout(ctx, pipeline_layout_ref.object());
        products.push(Product::new(
            CGenVariant::Hlsl,
            GeneratorContext::object_relative_path(pipeline_layout_ref.object(), CGenVariant::Hlsl),
            content.into_bytes(),
        ));
    }
    products
}

fn generate_hlsl_pipeline_layout(ctx: &GeneratorContext<'_>, pl: &PipelineLayout) -> String {
    let mut writer = FileWriter::new();

    // header
    {
        let mut writer = writer.add_block(
            &[
                format!("#ifndef PIPELINE_LAYOUT_{}", pl.name.to_uppercase()),
                format!("#define PIPELINE_LAYOUT_{}", pl.name.to_uppercase()),
            ],
            &["#endif"],
        );
        writer.new_line();

        writer.add_line("// DescriptorSets");
        for ds_handle in pl.descriptor_sets() {
            let ds = ds_handle.get(ctx.model);
            writer.add_lines(&[
                format!("// - name: {}", ds.name),
                format!("// - freq: {}", ds.frequency),
                format!(
                    "#include \"{}\"",
                    ctx.embedded_fs_path(ds, CGenVariant::Hlsl)
                ),
            ]);
        }
        writer.new_line();

        if let Some(pc_handle) = &pl.push_constant {
            let ty = pc_handle.get(ctx.model);
            writer.add_lines(&[
                "// - push constant",
                &format!(
                    "#include \"{}\"",
                    ctx.embedded_fs_path(ty, CGenVariant::Hlsl)
                ),
            ]);
            writer.add_lines(&[
                "[[vk::push_constant]]".to_string(),
                format!("ConstantBuffer<{}> push_constant;", ty.name()),
            ]);
        }

        // writer.add_line("// PushConstant".to_string());
        // for (name, ty) in &pl.members {
        //     match ty {
        //         crate::db::PipelineLayoutContent::PushConstant(ty_ref) => {
        //             let ty = ty_ref.get(ctx.model);
        //             writer.add_lines(&[
        //                 format!("// - name: {}", name),
        //                 format!(
        //                     "#include \"{}\"",
        //                     ctx.embedded_fs_path(ty, CGenVariant::Hlsl)
        //                 ),
        //             ]);
        //             writer.add_lines(&[
        //                 "[[vk::push_constant]]".to_string(),
        //                 format!("ConstantBuffer<{}> {}; ", ty.name(), name),
        //             ]);
        //         }
        //         crate::db::PipelineLayoutContent::DescriptorSet(_) => (),
        //     }
        // }
    }

    // finalize
    writer.build()
}
