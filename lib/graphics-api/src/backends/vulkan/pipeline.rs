#![allow(clippy::too_many_lines)]
use super::{
    VulkanApi, VulkanDeviceContext, VulkanRenderpassColorAttachment, VulkanRenderpassDef,
    VulkanRenderpassDepthAttachment, VulkanRootSignature,
};
use crate::{
    backends::deferred_drop::Drc, ComputePipelineDef, Format, GfxResult, GraphicsPipelineDef,
    LoadOp, Pipeline, PipelineType, ShaderStageFlags, StoreOp,
};
use ash::vk;
use std::ffi::CString;

#[derive(Debug)]
struct VulkanPipelineInner {
    pipeline_type: PipelineType,
    pipeline: vk::Pipeline,
    root_signature: VulkanRootSignature,
}

impl Drop for VulkanPipelineInner {
    fn drop(&mut self) {
        unsafe {
            let device = self.root_signature.device_context().device();
            device.destroy_pipeline(self.pipeline, None);
        }
    }
}

#[derive(Debug)]
pub struct VulkanPipeline {
    inner: Drc<VulkanPipelineInner>,
}

impl VulkanPipeline {
    pub fn vk_pipeline(&self) -> vk::Pipeline {
        self.inner.pipeline
    }

    pub fn new_graphics_pipeline(
        device_context: &VulkanDeviceContext,
        pipeline_def: &GraphicsPipelineDef<'_, VulkanApi>,
    ) -> GfxResult<Self> {
        //log::trace!("Create pipeline\n{:#?}", pipeline_def);

        //TODO: Cache
        let vk_root_signature = pipeline_def.root_signature;

        // image layouts and load/store ops don't affect compatibility
        // https://www.khronos.org/registry/vulkan/specs/1.1-extensions/html/chap9.html#renderpass-compatibility
        let color_attachments: Vec<_> = pipeline_def
            .color_formats
            .iter()
            .map(|&format| VulkanRenderpassColorAttachment {
                format,
                load_op: LoadOp::default(),
                store_op: StoreOp::default(),
            })
            .collect();

        let depth_attachment = if let Some(depth_format) = pipeline_def.depth_stencil_format {
            assert_ne!(depth_format, Format::UNDEFINED);
            Some(VulkanRenderpassDepthAttachment {
                format: depth_format,
                depth_load_op: LoadOp::default(),
                stencil_load_op: LoadOp::default(),
                depth_store_op: StoreOp::default(),
                stencil_store_op: StoreOp::default(),
            })
        } else {
            None
        };

        // Temporary renderpass, required to create pipeline but don't need to keep it
        let renderpass = device_context.create_renderpass(&VulkanRenderpassDef {
            color_attachments,
            depth_attachment,
        })?;

        let mut entry_point_names = vec![];
        for stage in pipeline_def.shader.stages() {
            entry_point_names.push(CString::new(stage.entry_point.clone()).unwrap());
        }

        let mut stages = vec![];
        for (stage, entry_point_name) in pipeline_def.shader.stages().iter().zip(&entry_point_names)
        {
            stages.push(
                vk::PipelineShaderStageCreateInfo::builder()
                    .name(entry_point_name)
                    .module(stage.shader_module.vk_shader_module())
                    .stage(stage.shader_stage.into())
                    .build(),
            );
        }

        let mut bindings = Vec::with_capacity(pipeline_def.vertex_layout.buffers.len());
        let mut attributes = Vec::with_capacity(pipeline_def.vertex_layout.attributes.len());

        for (index, vertex_buffer) in pipeline_def.vertex_layout.buffers.iter().enumerate() {
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
            .render_pass(renderpass.vk_renderpass())
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .build();

        //let depth = if pipeline_def.depth_stencil_format != Format::UNDEFINED {
        //    pipeline_def.depth_state.into_vk_builder();
        // } else {
        //     let depth_state = DepthState::default();
        //     depth_state.into_vk_builder()
        // };

        let pipeline = unsafe {
            match device_context.device().create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_create_info],
                None,
            ) {
                Ok(result) => Ok(result),
                Err(e) => Err(e.1),
            }
        }?[0];

        Ok(Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(VulkanPipelineInner {
                    pipeline_type: PipelineType::Graphics,
                    pipeline,
                    root_signature: pipeline_def.root_signature.clone(),
                }),
        })
    }

    pub fn new_compute_pipeline(
        device_context: &VulkanDeviceContext,
        pipeline_def: &ComputePipelineDef<'_, VulkanApi>,
    ) -> GfxResult<Self> {
        //log::trace!("Create pipeline\n{:#?}", pipeline_def);

        //TODO: Cache
        let vk_root_signature = pipeline_def.root_signature;

        let vk_shader = pipeline_def.shader;
        assert_eq!(vk_shader.stages().len(), 1);
        assert_eq!(vk_shader.stage_flags(), ShaderStageFlags::COMPUTE);

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

        let pipeline = unsafe {
            match device_context.device().create_compute_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_create_info],
                None,
            ) {
                Ok(result) => Ok(result),
                Err(e) => Err(e.1),
            }
        }?[0];

        Ok(Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(VulkanPipelineInner {
                    pipeline_type: PipelineType::Compute,
                    pipeline,
                    root_signature: pipeline_def.root_signature.clone(),
                }),
        })
    }
}

impl Pipeline<VulkanApi> for VulkanPipeline {
    fn pipeline_type(&self) -> PipelineType {
        self.inner.pipeline_type
    }

    fn root_signature(&self) -> &VulkanRootSignature {
        &self.inner.root_signature
    }
}
