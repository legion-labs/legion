use lgn_graphics_api::MAX_DESCRIPTOR_SET_LAYOUTS;

use crate::{
    db::PipelineLayout,
    generators::{file_writer::FileWriter, product::Product, CGenVariant, GeneratorContext},
};

pub fn run(ctx: &GeneratorContext<'_>) -> Vec<Product> {
    let mut products = Vec::new();
    let model = ctx.model;
    for pipeline_layout_ref in model.object_iter::<PipelineLayout>() {
        let content = generate_rust_pipeline_layout(
            ctx,
            pipeline_layout_ref.id(),
            pipeline_layout_ref.object(),
        );
        products.push(Product::new(
            CGenVariant::Rust,
            GeneratorContext::object_relative_path(pipeline_layout_ref.object(), CGenVariant::Rust),
            content.into_bytes(),
        ));
    }

    products
}

fn generate_rust_pipeline_layout(
    ctx: &GeneratorContext<'_>,
    pipeline_layout_id: u32,
    pipeline_layout: &PipelineLayout,
) -> String {
    let mut writer = FileWriter::new();

    // global dependencies
    writer.add_line("use std::{mem, ptr};");
    writer.new_line();

    {
        let mut writer = writer.add_block(&["use lgn_graphics_api::{"], &["};"]);
        writer.add_lines(&[
            "DeviceContext,",
            "RootSignature,",
            "DescriptorSetLayout,",
            "DescriptorSetHandle,",
            "MAX_DESCRIPTOR_SET_LAYOUTS,",
        ]);
    }
    writer.new_line();

    {
        let mut writer = writer.add_block(&["use lgn_graphics_cgen_runtime::{"], &["};"]);
        writer.add_lines(&["CGenPipelineLayoutDef,", "PipelineDataProvider,"]);
    }
    writer.new_line();

    // local dependencies
    {
        for (_, content) in &pipeline_layout.members {
            match content {
                crate::db::PipelineLayoutContent::DescriptorSet(ds_ref) => {
                    let ds = ds_ref.get(ctx.model);
                    writer.add_line(format!("use super::super::descriptor_set::{};", ds.name));
                }
                crate::db::PipelineLayoutContent::PushConstant(ty_ref) => {
                    let ty = ty_ref.get(ctx.model);
                    writer.add_line(format!("use super::super::cgen_type::{};", ty.name()));
                }
            }
        }
    }
    writer.new_line();

    // write cgen pipeline layout def
    {
        let mut writer = writer.add_block(
            &["static PIPELINE_LAYOUT_DEF: CGenPipelineLayoutDef = CGenPipelineLayoutDef{"],
            &["};"],
        );
        writer.add_line(format!("name: \"{}\",", pipeline_layout.name));
        writer.add_line(format!("id: {},", pipeline_layout_id));
        writer.add_line("descriptor_set_layout_ids: [");
        for i in 0..MAX_DESCRIPTOR_SET_LAYOUTS {
            let opt_ds_ref = pipeline_layout.find_descriptor_set_by_frequency(ctx.model, i);
            match opt_ds_ref {
                Some(ds_ref) => {
                    let ds = ds_ref.get(ctx.model);
                    writer.add_line(format!("Some({}::id()),", ds.name));
                }
                None => writer.add_line("None,"),
            }
        }
        writer.add_line("],");
        if let Some(ty_handle) = pipeline_layout.push_constant() {
            let ty = ty_handle.get(ctx.model);
            writer.add_line(format!("push_constant_type: Some({}::id())", ty.name()));
        } else {
            writer.add_line("push_constant_type: None,");
        }
    }

    writer.new_line();

    // pipeline layout
    {
        writer.add_line("static mut PIPELINE_LAYOUT: Option<RootSignature> = None;");
    }

    writer.new_line();

    // struct
    {
        let mut writer =
            writer.add_block(&[format!("pub struct {} {{", pipeline_layout.name)], &["}"]);

        writer.add_line(
            "descriptor_sets: [Option<DescriptorSetHandle>; MAX_DESCRIPTOR_SET_LAYOUTS],",
        );
        if let Some(ty_ref) = pipeline_layout.push_constant() {
            let ty = ty_ref.get(ctx.model);
            writer.add_line(format!("push_constant: {}", ty.name()));
        }
    }

    writer.new_line();

    // impl
    {
        let mut writer = writer.add_block(&[format!("impl {} {{", pipeline_layout.name)], &["}"]);
        // fn initialize
        {
            let mut writer = writer.add_block(
                &["#[allow(unsafe_code)]", "pub fn initialize(device_context: &DeviceContext, descriptor_set_layouts: &[&DescriptorSetLayout]) {"],
                &["}"],
            );
            writer.add_line("unsafe { ");
            if let Some(ty_handle) = pipeline_layout.push_constant() {
                let ty = ty_handle.get(ctx.model);
                writer.add_line(format!(
                    "let push_constant_def = Some({}::def());",
                    ty.name()
                ));
            } else {
                writer.add_line("let push_constant_def = None");
            };
            writer.add_lines(&["PIPELINE_LAYOUT = Some(PIPELINE_LAYOUT_DEF.create_pipeline_layout(device_context, descriptor_set_layouts, push_constant_def));", "}"]);
        }
        writer.new_line();

        // fn shutdown
        {
            let mut writer =
                writer.add_block(&["#[allow(unsafe_code)]", "pub fn shutdown() {"], &["}"]);
            writer.add_line("unsafe{ PIPELINE_LAYOUT = None; }");
        }
        writer.new_line();

        // fn root_signature
        {
            let mut writer = writer.add_block(
                &[
                    "#[allow(unsafe_code)]",
                    "pub fn root_signature() -> &'static RootSignature {",
                ],
                &["}"],
            );
            {
                let mut writer = writer.add_block(&["unsafe{ match &PIPELINE_LAYOUT{"], &["}}"]);
                writer.add_line("Some(pl) => pl,");
                writer.add_line("None => unreachable!(),");
            }
        }
        writer.new_line();

        // fn new
        {
            let mut writer = writer.add_block(&["pub fn new() -> Self {"], &["}"]);
            writer.add_line("Self::default()");
        }
        writer.new_line();

        // fn setters
        for (name, content) in &pipeline_layout.members {
            match content {
                crate::db::PipelineLayoutContent::DescriptorSet(ds_ref) => {
                    let ds = ds_ref.get(ctx.model);
                    let mut writer = writer.add_block(
                        &[format!(
                        "pub fn set_{}(&mut self, descriptor_set_handle: DescriptorSetHandle) {{",
                        name
                    )],
                        &["}"],
                    );

                    writer.add_line(format!(
                        "self.descriptor_sets[{}] = Some(descriptor_set_handle);",
                        ds.frequency
                    ));
                }
                crate::db::PipelineLayoutContent::PushConstant(ty_ref) => {
                    let ty = ty_ref.get(ctx.model);
                    let mut writer = writer.add_block(
                        &[format!(
                            "pub fn set_{}(&mut self, data: &{}) {{",
                            name,
                            ty.name()
                        )],
                        &["}"],
                    );

                    writer.add_line("self.push_constant = *data;");
                }
            }
        }
    }

    writer.new_line();

    // trait: Default
    {
        let mut writer = writer.add_block(
            &[format!("impl Default for {} {{", pipeline_layout.name)],
            &["}"],
        );

        {
            let mut writer = writer.add_block(&["fn default() -> Self {"], &["}"]);
            {
                let mut writer = writer.add_block(&["Self {"], &["}"]);
                writer.add_line("descriptor_sets: [None; MAX_DESCRIPTOR_SET_LAYOUTS],");
                if let Some(ty_ref) = pipeline_layout.push_constant() {
                    let ty = ty_ref.get(ctx.model);
                    writer.add_line(format!("push_constant: {}::default(),", ty.name()));
                }
            }
        }
    }

    writer.new_line();

    // trait: PipelineDataProvider
    {
        let mut writer = writer.add_block(
            &[format!(
                "impl PipelineDataProvider for {} {{",
                pipeline_layout.name
            )],
            &["}"],
        );
        writer.new_line();

        // fn descriptor_set
        {
            let mut writer =
                writer.add_block(&["fn root_signature() -> &'static RootSignature {"], &["}"]);
            writer.add_line("Self::root_signature()");
        }
        writer.new_line();

        // fn descriptor_set
        {
            let mut writer = writer.add_block(
                &["fn descriptor_set(&self, frequency: u32) -> Option<DescriptorSetHandle> {"],
                &["}"],
            );
            writer.add_line("self.descriptor_sets[frequency as usize]");
        }
        writer.new_line();

        // fn push_constant
        {
            let mut writer =
                writer.add_block(&["fn push_constant(&self) -> Option<&[u8]> {"], &["}"]);
            if let Some(ty_handle) = pipeline_layout.push_constant() {
                writer.add_line("#![allow(unsafe_code)]");
                let ty = ty_handle.get(ctx.model);
                writer.add_line("let data_slice = unsafe {");
                writer.add_line(format!("&*ptr::slice_from_raw_parts((&self.push_constant as *const {0}).cast::<u8>(), mem::size_of::<{0}>())", ty.name()));
                writer.add_line("};");
                writer.add_line("Some(data_slice)");
            } else {
                writer.add_line("None");
            }
        }
    }
    writer.new_line();

    // finalize
    writer.build()
}
