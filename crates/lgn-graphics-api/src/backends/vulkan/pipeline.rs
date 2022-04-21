use std::ffi::CString;

use ash::vk;

use crate::{
    ComputePipelineDef, DeviceContext, GfxResult, GraphicsPipelineDef, Pipeline, ShaderStageFlags,
};

#[derive(Debug)]
pub(crate) struct VulkanPipeline {
    vk_pipeline: vk::Pipeline,
}

impl VulkanPipeline {
    pub fn new_graphics_pipeline(
        device_context: &DeviceContext,
        pipeline_def: &GraphicsPipelineDef<'_>,
    ) -> GfxResult<Self> {
        //log::trace!("Create pipeline\n{:#?}", pipeline_def);

        //TODO: Cache
        let vk_root_signature = pipeline_def.root_signature;

        let mut vk_color_formats: Vec<ash::vk::Format> =
            vec![ash::vk::Format::UNDEFINED; pipeline_def.color_formats.len()];
        for (i, format) in pipeline_def.color_formats.iter().enumerate() {
            vk_color_formats[i] = ash::vk::Format::from(*format);
        }

        let vk_depth_format = if let Some(depth_format) = pipeline_def.depth_stencil_format {
            depth_format.into()
        } else {
            ash::vk::Format::UNDEFINED
        };

        let mut pipeline_rendering_create_info = vk::PipelineRenderingCreateInfo::builder()
            .color_attachment_formats(&vk_color_formats)
            .depth_attachment_format(vk_depth_format)
            .stencil_attachment_format(vk_depth_format)
            .build();

        let mut entry_point_names = vec![];
        for stage in pipeline_def.shader.stages() {
            entry_point_names.push(CString::new(stage.entry_point.clone()).unwrap());
        }

        let mut stages = vec![];
        for (stage, entry_point_name) in pipeline_def.shader.stages().iter().zip(&entry_point_names)
        {
            let shader_stage_flags: ShaderStageFlags = stage.shader_stage.into();
            stages.push(
                vk::PipelineShaderStageCreateInfo::builder()
                    .name(entry_point_name)
                    .module(stage.shader_module.vk_shader_module())
                    .stage(shader_stage_flags.into())
                    .build(),
            );
        }

        let mut bindings = Vec::with_capacity(pipeline_def.vertex_layout.buffers.len());
        let mut attributes = Vec::with_capacity(pipeline_def.vertex_layout.attributes.len());

        for (index, vertex_buffer) in pipeline_def.vertex_layout.buffers.iter().enumerate() {
            if vertex_buffer.is_none() {
                break;
            }
            let vertex_buffer = vertex_buffer.as_ref().unwrap();
            if vertex_buffer.stride > 0 {
                bindings.push(
                    vk::VertexInputBindingDescription::builder()
                        .binding(index as u32)
                        .input_rate(vertex_buffer.rate.into())
                        .stride(vertex_buffer.stride)
                        .build(),
                );
            }
        }

        for vertex_attribute in &pipeline_def.vertex_layout.attributes {
            if vertex_attribute.is_none() {
                break;
            }
            let vertex_attribute = vertex_attribute.as_ref().unwrap();
            attributes.push(
                vk::VertexInputAttributeDescription::builder()
                    .format(vertex_attribute.format.into())
                    .location(vertex_attribute.location)
                    .binding(vertex_attribute.buffer_index)
                    .offset(vertex_attribute.byte_offset)
                    .build(),
            );
        }

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&bindings)
            .vertex_attribute_descriptions(&attributes);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(pipeline_def.primitive_topology.into())
            .primitive_restart_enable(false);

        // Tesselation not supported

        // Set up for dynamic viewport/scissor
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(pipeline_def.sample_count.into())
            .sample_shading_enable(false)
            .min_sample_shading(0.0)
            .alpha_to_coverage_enable(false) // pipeline_def.blend_state.alpha_to_coverage_enable?
            .alpha_to_one_enable(false);

        let rasterization_state =
            super::internal::rasterizer_state_to_create_info(pipeline_def.rasterizer_state);
        let depth_state = super::internal::depth_state_to_create_info(pipeline_def.depth_state);
        let blend_state = super::internal::blend_state_to_create_info(
            pipeline_def.blend_state,
            pipeline_def.color_formats.len(),
        );

        let dynamic_states = [
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR,
            vk::DynamicState::DEPTH_BIAS,
            vk::DynamicState::BLEND_CONSTANTS,
            vk::DynamicState::DEPTH_BOUNDS,
            vk::DynamicState::STENCIL_REFERENCE,
        ];
        let dynamic_states_create_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_state)
            .color_blend_state(blend_state.blend_state())
            .dynamic_state(&dynamic_states_create_info)
            .layout(vk_root_signature.vk_pipeline_layout())
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .push_next(&mut pipeline_rendering_create_info)
            .build();

        //let depth = if pipeline_def.depth_stencil_format != Format::UNDEFINED {
        //    pipeline_def.depth_state.into_vk_builder();
        // } else {
        //     let depth_state = DepthState::default();
        //     depth_state.into_vk_builder()
        // };

        let vk_pipeline = unsafe {
            match device_context.vk_device().create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_create_info],
                None,
            ) {
                Ok(result) => Ok(result),
                Err(e) => Err(e.1),
            }
        }?[0];

        Ok(Self { vk_pipeline })
    }

    pub fn new_compute_pipeline(
        device_context: &DeviceContext,
        pipeline_def: &ComputePipelineDef<'_>,
    ) -> GfxResult<Self> {
        //log::trace!("Create pipeline\n{:#?}", pipeline_def);

        //TODO: Cache
        let vk_root_signature = pipeline_def.root_signature;

        let vk_shader = pipeline_def.shader;
        assert_eq!(vk_shader.stages().len(), 1);
        assert_eq!(vk_shader.stage_flags(), ShaderStageFlags::COMPUTE_FLAG);

        let mut entry_point_names = vec![];
        for stage in vk_shader.stages() {
            entry_point_names.push(CString::new(stage.entry_point.clone()).unwrap());
        }

        let compute_stage = &vk_shader.stages()[0];
        let entry_point_name = CString::new(compute_stage.entry_point.clone()).unwrap();
        let stage = vk::PipelineShaderStageCreateInfo::builder()
            .name(&entry_point_name)
            .module(compute_stage.shader_module.vk_shader_module())
            .stage(vk::ShaderStageFlags::COMPUTE);

        let pipeline_create_info = vk::ComputePipelineCreateInfo::builder()
            .stage(*stage)
            .layout(vk_root_signature.vk_pipeline_layout())
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .build();

        let vk_pipeline = unsafe {
            match device_context.vk_device().create_compute_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_create_info],
                None,
            ) {
                Ok(result) => Ok(result),
                Err(e) => Err(e.1),
            }
        }?[0];

        Ok(Self { vk_pipeline })
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        unsafe {
            device_context
                .vk_device()
                .destroy_pipeline(self.vk_pipeline, None);
        }
    }
}

impl Pipeline {
    pub fn vk_pipeline(&self) -> vk::Pipeline {
        self.inner.backend_pipeline.vk_pipeline
    }
}
