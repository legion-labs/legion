// This is generated file. Do not edit manually

#![allow(clippy::all)]
#![allow(dead_code)]

use lgn_graphics_api::DeviceContext;
pub mod cgen_type;
pub mod descriptor_set;
pub mod pipeline_layout;

pub fn initialize(device_context: &DeviceContext) {
    descriptor_set::FrameDescriptorSet::initialize(device_context);
    descriptor_set::ViewDescriptorSet::initialize(device_context);
    descriptor_set::EguiDescriptorSet::initialize(device_context);
    descriptor_set::PickingDescriptorSet::initialize(device_context);

    let descriptor_set_layouts = [
        descriptor_set::FrameDescriptorSet::descriptor_set_layout(),
        descriptor_set::ViewDescriptorSet::descriptor_set_layout(),
        descriptor_set::EguiDescriptorSet::descriptor_set_layout(),
        descriptor_set::PickingDescriptorSet::descriptor_set_layout(),
    ];

    pipeline_layout::EguiPipelineLayout::initialize(device_context, &descriptor_set_layouts);
    pipeline_layout::ConstColorPipelineLayout::initialize(device_context, &descriptor_set_layouts);
    pipeline_layout::PickingPipelineLayout::initialize(device_context, &descriptor_set_layouts);
    pipeline_layout::ShaderPipelineLayout::initialize(device_context, &descriptor_set_layouts);
}

pub fn shutdown() {
    descriptor_set::FrameDescriptorSet::shutdown();
    descriptor_set::ViewDescriptorSet::shutdown();
    descriptor_set::EguiDescriptorSet::shutdown();
    descriptor_set::PickingDescriptorSet::shutdown();

    pipeline_layout::EguiPipelineLayout::shutdown();
    pipeline_layout::ConstColorPipelineLayout::shutdown();
    pipeline_layout::PickingPipelineLayout::shutdown();
    pipeline_layout::ShaderPipelineLayout::shutdown();
}

#[rustfmt::skip]
mod shader_files {
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static OMNI_DIRECTIONAL_LIGHT: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/cgen_type/omni_directional_light.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/cgen_type/omni_directional_light.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static DIRECTIONAL_LIGHT: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/cgen_type/directional_light.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/cgen_type/directional_light.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static SPOT_LIGHT: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/cgen_type/spot_light.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/cgen_type/spot_light.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static VIEW_DATA: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/cgen_type/view_data.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/cgen_type/view_data.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static LIGHTING_DATA: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/cgen_type/lighting_data.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/cgen_type/lighting_data.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static ENTITY_TRANSFORMS: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/cgen_type/entity_transforms.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/cgen_type/entity_transforms.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static EGUI_PUSH_CONSTANT_DATA: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/cgen_type/egui_push_constant_data.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/cgen_type/egui_push_constant_data.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static CONST_COLOR_PUSH_CONSTANT_DATA: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/cgen_type/const_color_push_constant_data.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/cgen_type/const_color_push_constant_data.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static PICKING_DATA: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/cgen_type/picking_data.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/cgen_type/picking_data.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static PICKING_PUSH_CONSTANT_DATA: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/cgen_type/picking_push_constant_data.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/cgen_type/picking_push_constant_data.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static INSTANCE_PUSH_CONSTANT_DATA: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/cgen_type/instance_push_constant_data.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/cgen_type/instance_push_constant_data.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static EGUI_PIPELINE_LAYOUT: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/pipeline_layout/egui_pipeline_layout.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/pipeline_layout/egui_pipeline_layout.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static CONST_COLOR_PIPELINE_LAYOUT: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/pipeline_layout/const_color_pipeline_layout.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/pipeline_layout/const_color_pipeline_layout.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static PICKING_PIPELINE_LAYOUT: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/pipeline_layout/picking_pipeline_layout.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/pipeline_layout/picking_pipeline_layout.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static SHADER_PIPELINE_LAYOUT: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/pipeline_layout/shader_pipeline_layout.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/pipeline_layout/shader_pipeline_layout.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static FRAME_DESCRIPTOR_SET: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/descriptor_set/frame_descriptor_set.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/descriptor_set/frame_descriptor_set.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static VIEW_DESCRIPTOR_SET: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/descriptor_set/view_descriptor_set.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/descriptor_set/view_descriptor_set.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static EGUI_DESCRIPTOR_SET: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/descriptor_set/egui_descriptor_set.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/descriptor_set/egui_descriptor_set.hlsl")),
        None
    );
    #[linkme::distributed_slice(lgn_embedded_fs::EMBEDDED_FILES)]
    static PICKING_DESCRIPTOR_SET: lgn_embedded_fs::EmbeddedFile = lgn_embedded_fs::EmbeddedFile::new(
        "crate://lgn-renderer/codegen/hlsl/descriptor_set/picking_descriptor_set.hlsl",
        include_bytes!(concat!(env!("OUT_DIR"), "/codegen/hlsl/descriptor_set/picking_descriptor_set.hlsl")),
        None
    );
}
