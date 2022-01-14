#![allow(clippy::too_many_lines)]

use lgn_graphics_api::{prelude::*, MAX_DESCRIPTOR_SET_LAYOUTS};
use lgn_pso_compiler::{CompileParams, EntryPoint, HlslCompiler, ShaderSource, TargetProfile};
use lgn_renderer::{
    components::{RenderSurface, RenderSurfaceExtents},
    RenderContext,
};
use lgn_tracing::span_fn;

pub struct OffscreenHelper {
    render_image: Texture,
    render_image_rtv: TextureView,
    copy_image: Texture,
    root_signature: RootSignature,
    pipeline: Pipeline,
    bilinear_sampler: Sampler,
}

impl OffscreenHelper {
    pub fn new(
        shader_compiler: &HlslCompiler,
        device_context: &DeviceContext,
        resolution: RenderSurfaceExtents,
    ) -> anyhow::Result<Self> {
        shader_compiler
            .filesystem()
            .add_mount_point("presenter-snapshot", env!("CARGO_MANIFEST_DIR"))?;

        //
        // Immutable resources
        //
        let shader_build_result = shader_compiler.compile(&CompileParams {
            shader_source: ShaderSource::Path(
                "crate://presenter-snapshot/shaders/display_mapper.hlsl".to_string(),
            ),
            global_defines: Vec::new(),
            entry_points: vec![
                EntryPoint {
                    defines: Vec::new(),
                    name: "main_vs".to_owned(),
                    target_profile: TargetProfile::Vertex,
                },
                EntryPoint {
                    defines: Vec::new(),
                    name: "main_ps".to_owned(),
                    target_profile: TargetProfile::Pixel,
                },
            ],
        })?;

        let vert_shader_module = device_context.create_shader_module(
            ShaderPackage::SpirV(shader_build_result.spirv_binaries[0].bytecode.clone())
                .module_def(),
        )?;

        let frag_shader_module = device_context.create_shader_module(
            ShaderPackage::SpirV(shader_build_result.spirv_binaries[1].bytecode.clone())
                .module_def(),
        )?;

        let shader = device_context.create_shader(vec![
            ShaderStageDef {
                entry_point: "main_vs".to_owned(),
                shader_stage: ShaderStageFlags::VERTEX,
                shader_module: vert_shader_module,
                // reflection: shader_build_result.reflection_info.clone().unwrap(),
            },
            ShaderStageDef {
                entry_point: "main_ps".to_owned(),
                shader_stage: ShaderStageFlags::FRAGMENT,
                shader_module: frag_shader_module,
                // reflection: shader_build_result.reflection_info.clone().unwrap(),
            },
        ]);

        let mut descriptor_set_layouts = Vec::new();
        for set_index in 0..MAX_DESCRIPTOR_SET_LAYOUTS {
            let shader_resources: Vec<_> = shader_build_result
                .pipeline_reflection
                .shader_resources
                .iter()
                .filter(|x| x.set_index as usize == set_index)
                .collect();

            if !shader_resources.is_empty() {
                let descriptor_defs = shader_resources
                    .iter()
                    .map(|sr| DescriptorDef {
                        name: sr.name.clone(),
                        binding: sr.binding,
                        shader_resource_type: sr.shader_resource_type,
                        array_size: sr.element_count,
                    })
                    .collect();

                let def = DescriptorSetLayoutDef {
                    frequency: set_index as u32,
                    descriptor_defs,
                };
                let descriptor_set_layout =
                    device_context.create_descriptorset_layout(&def).unwrap();
                descriptor_set_layouts.push(descriptor_set_layout);
            }
        }

        let root_signature_def = RootSignatureDef {
            descriptor_set_layouts: descriptor_set_layouts.clone(),
            push_constant_def: None,
        };

        let root_signature = device_context.create_root_signature(&root_signature_def)?;

        let pipeline = device_context.create_graphics_pipeline(&GraphicsPipelineDef {
            shader: &shader,
            root_signature: &root_signature,
            vertex_layout: &VertexLayout::default(),
            blend_state: &BlendState::default(),
            depth_state: &DepthState::default(),
            rasterizer_state: &RasterizerState {
                cull_mode: CullMode::Back,
                ..RasterizerState::default()
            },
            primitive_topology: PrimitiveTopology::TriangleList,
            color_formats: &[Format::R8G8B8A8_UNORM],
            depth_stencil_format: None,
            sample_count: SampleCount::SampleCount1,
        })?;

        let sampler_def = SamplerDef {
            min_filter: FilterType::Linear,
            mag_filter: FilterType::Linear,
            mip_map_mode: MipMapMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            ..SamplerDef::default()
        };
        let bilinear_sampler = device_context.create_sampler(&sampler_def)?;

        let render_image = device_context.create_texture(&TextureDef {
            extents: Extents3D {
                width: resolution.width(),
                height: resolution.height(),
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            mem_usage: MemoryUsage::GpuOnly,
            usage_flags: ResourceUsage::AS_RENDER_TARGET | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            tiling: TextureTiling::Optimal,
        })?;

        let render_image_rtv = render_image.create_view(&TextureViewDef::as_render_target_view(
            render_image.definition(),
        ))?;

        let copy_image = device_context.create_texture(&TextureDef {
            extents: Extents3D {
                width: resolution.width(),
                height: resolution.height(),
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            mem_usage: MemoryUsage::GpuToCpu,
            usage_flags: ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            tiling: TextureTiling::Linear,
        })?;

        Ok(Self {
            render_image,
            render_image_rtv,
            copy_image,
            root_signature,
            pipeline,
            bilinear_sampler,
        })
    }

    #[span_fn]
    pub fn present<F: FnOnce(&[u8], usize)>(
        &mut self,
        render_context: &RenderContext<'_>,
        render_surface: &mut RenderSurface,
        copy_fn: F,
    ) -> anyhow::Result<()> {
        let cmd_buffer = render_context.alloc_command_buffer();
        let render_texture = &self.render_image;
        let render_texture_rtv = &self.render_image_rtv;
        let copy_texture = &self.copy_image;

        render_surface.transition_to(&cmd_buffer, ResourceState::SHADER_RESOURCE);

        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                render_texture,
                ResourceState::COPY_SRC,
                ResourceState::RENDER_TARGET,
            )],
        );

        cmd_buffer.begin_render_pass(
            &[ColorRenderTargetBinding {
                texture_view: render_texture_rtv,
                load_op: LoadOp::DontCare,
                store_op: StoreOp::Store,
                clear_value: ColorClearValue::default(),
            }],
            &None,
        );

        cmd_buffer.bind_pipeline(&self.pipeline);

        let descriptor_set_layout = &self
            .pipeline
            .root_signature()
            .definition()
            .descriptor_set_layouts[0];
        let mut descriptor_set_writer = render_context.alloc_descriptor_set(descriptor_set_layout);
        descriptor_set_writer
            .set_descriptors_by_name(
                "hdr_image",
                &[DescriptorRef::TextureView(
                    render_surface.shader_resource_view(),
                )],
            )
            .unwrap();
        descriptor_set_writer
            .set_descriptors_by_name(
                "hdr_sampler",
                &[DescriptorRef::Sampler(&self.bilinear_sampler)],
            )
            .unwrap();

        let device_context = render_context.renderer().device_context();
        let descriptor_set_handle = descriptor_set_writer.flush(device_context);

        cmd_buffer.bind_descriptor_set_handle(
            PipelineType::Graphics,
            &self.root_signature,
            descriptor_set_layout.definition().frequency,
            descriptor_set_handle,
        );

        cmd_buffer.draw(3, 0);

        cmd_buffer.end_render_pass();

        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                render_texture,
                ResourceState::RENDER_TARGET,
                ResourceState::COPY_SRC,
            )],
        );

        //
        // Copy
        //

        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                copy_texture,
                ResourceState::COMMON,
                ResourceState::COPY_DST,
            )],
        );

        let copy_extents = render_texture.definition().extents;
        assert_eq!(copy_texture.definition().extents, copy_extents);

        cmd_buffer.copy_image(
            render_texture,
            copy_texture,
            &CmdCopyTextureParams {
                src_state: ResourceState::COPY_SRC,
                dst_state: ResourceState::COPY_DST,
                src_offset: Offset3D { x: 0, y: 0, z: 0 },
                dst_offset: Offset3D { x: 0, y: 0, z: 0 },
                src_mip_level: 0,
                dst_mip_level: 0,
                src_array_slice: 0,
                dst_array_slice: 0,
                extent: copy_extents,
                src_plane_slice: PlaneSlice::Default,
                dst_plane_slice: PlaneSlice::Default,
            },
        );

        cmd_buffer.resource_barrier(
            &[],
            &[TextureBarrier::state_transition(
                copy_texture,
                ResourceState::COPY_DST,
                ResourceState::COMMON,
            )],
        );

        //
        // Present the image
        //

        let wait_sem = render_surface.sema();
        let graphics_queue = render_context.graphics_queue();

        graphics_queue.submit(&mut [cmd_buffer.finalize()], &[wait_sem], &[], None);

        graphics_queue.wait_for_queue_idle()?;

        let sub_resource = copy_texture.map_texture(PlaneSlice::Default)?;
        copy_fn(sub_resource.data, sub_resource.row_pitch as usize);
        copy_texture.unmap_texture();
        Ok(())
    }
}
