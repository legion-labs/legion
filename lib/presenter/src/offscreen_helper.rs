#![allow(clippy::too_many_lines)]

use legion_graphics_api::{prelude::*, MAX_DESCRIPTOR_SET_LAYOUTS};
use legion_pso_compiler::{CompileParams, EntryPoint, HlslCompiler, ShaderSource};
use legion_renderer::components::RenderSurface;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    pub fn new(mut width: u32, mut height: u32) -> Self {
        // Ensure a minimum size for the resolution.
        if width < 16 {
            width = 16;
        }

        if height < 16 {
            height = 16;
        }

        Self {
            // Make sure width & height always are multiple of 2.
            width: width & !1,
            height: height & !1,
        }
    }

    pub fn width(self) -> u32 {
        self.width
    }

    pub fn height(self) -> u32 {
        self.height
    }
}

struct ResolutionDependentResources {
    resolution: Resolution,
    render_images: Vec<<DefaultApi as GfxApi>::Texture>,
    render_image_rtvs: Vec<<DefaultApi as GfxApi>::TextureView>,
    copy_images: Vec<<DefaultApi as GfxApi>::Texture>,
}

impl ResolutionDependentResources {
    fn new(
        device_context: &<DefaultApi as GfxApi>::DeviceContext,
        render_frame_count: u32,
        resolution: Resolution,
    ) -> Result<Self, anyhow::Error> {
        let mut render_images = Vec::with_capacity(render_frame_count as usize);
        let mut render_image_rtvs = Vec::with_capacity(render_frame_count as usize);
        let mut copy_images = Vec::with_capacity(render_frame_count as usize);
        for _ in 0..render_frame_count {
            let render_image = device_context.create_texture(&TextureDef {
                extents: Extents3D {
                    width: resolution.width,
                    height: resolution.height,
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

            let render_image_rtv = render_image.create_view(
                &TextureViewDef::as_render_target_view(render_image.definition()),
            )?;

            let copy_image = device_context.create_texture(&TextureDef {
                extents: Extents3D {
                    width: resolution.width,
                    height: resolution.height,
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

            render_images.push(render_image);
            render_image_rtvs.push(render_image_rtv);
            copy_images.push(copy_image);
        }

        Ok(Self {
            resolution,
            render_images,
            render_image_rtvs,
            copy_images,
        })
    }
}

pub struct OffscreenHelper {
    render_frame_count: u32,
    resolution_dependent_resources: ResolutionDependentResources,
    cmd_pools: Vec<<DefaultApi as GfxApi>::CommandPool>,
    cmd_buffers: Vec<<DefaultApi as GfxApi>::CommandBuffer>,
    root_signature: <DefaultApi as GfxApi>::RootSignature,
    pipeline: <DefaultApi as GfxApi>::Pipeline,
    descriptor_set_arrays: Vec<<DefaultApi as GfxApi>::DescriptorSetArray>,
    bilinear_sampler: <DefaultApi as GfxApi>::Sampler,
}

impl OffscreenHelper {
    pub fn new(
        device_context: &<DefaultApi as GfxApi>::DeviceContext,
        graphics_queue: &<DefaultApi as GfxApi>::Queue,
        resolution: Resolution,
    ) -> anyhow::Result<Self> {
        //
        // Immutable resources
        //
        let shader_compiler = HlslCompiler::new().unwrap();

        let shader_source =
            String::from_utf8(include_bytes!("../data/display_mapper.hlsl").to_vec())?;

        let shader_build_result = shader_compiler.compile(&CompileParams {
            shader_source: ShaderSource::Code(shader_source),
            glob_defines: Vec::new(),
            entry_points: vec![
                EntryPoint {
                    defines: Vec::new(),
                    name: "main_vs".to_owned(),
                    target_profile: "vs_6_0".to_owned(),
                },
                EntryPoint {
                    defines: Vec::new(),
                    name: "main_ps".to_owned(),
                    target_profile: "ps_6_0".to_owned(),
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

        let shader = device_context.create_shader(
            vec![
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
            ],
            &shader_build_result.pipeline_reflection,
        )?;

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
            pipeline_type: PipelineType::Graphics,
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

        //
        // Frame dependant resources
        //
        let render_frame_count = 1u32;

        let resolution_dependent_resources =
            ResolutionDependentResources::new(device_context, render_frame_count, resolution)?;

        let mut cmd_pools = Vec::with_capacity(render_frame_count as usize);
        let mut cmd_buffers = Vec::with_capacity(render_frame_count as usize);

        for _ in 0..render_frame_count {
            let cmd_pool =
                graphics_queue.create_command_pool(&CommandPoolDef { transient: true })?;

            let cmd_buffer = cmd_pool.create_command_buffer(&CommandBufferDef {
                is_secondary: false,
            })?;

            cmd_pools.push(cmd_pool);
            cmd_buffers.push(cmd_buffer);
        }

        let heap_def = DescriptorHeapDef::from_descriptor_set_layout_def(
            descriptor_set_layouts[0].definition(),
            false,
            render_frame_count,
        );
        let descriptor_heap = device_context.create_descriptor_heap(&heap_def).unwrap();

        let mut descriptor_set_arrays = Vec::new();
        for descriptor_set_layout in &root_signature_def.descriptor_set_layouts {
            let descriptor_set_array = device_context
                .create_descriptor_set_array(
                    descriptor_heap.clone(),
                    &DescriptorSetArrayDef {
                        descriptor_set_layout,
                        array_length: render_frame_count,
                    },
                )
                .unwrap();
            descriptor_set_arrays.push(descriptor_set_array);
        }

        Ok(Self {
            render_frame_count: render_frame_count as u32,
            resolution_dependent_resources,
            cmd_pools,
            cmd_buffers,
            root_signature,
            pipeline,
            descriptor_set_arrays,
            bilinear_sampler,
        })
    }

    pub fn resize(
        &mut self,
        device_context: &<DefaultApi as GfxApi>::DeviceContext,
        resolution: Resolution,
    ) -> anyhow::Result<bool> {
        if resolution != self.resolution_dependent_resources.resolution {
            self.resolution_dependent_resources = ResolutionDependentResources::new(
                device_context,
                self.render_frame_count,
                resolution,
            )?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn present<F: FnOnce(&[u8], usize)>(
        &mut self,
        graphics_queue: &<DefaultApi as GfxApi>::Queue,
        wait_sem: &<DefaultApi as GfxApi>::Semaphore,
        render_surface: &mut RenderSurface,
        copy_fn: F,
    ) -> anyhow::Result<()> {
        //
        // Render
        //
        let render_frame_idx = 0;
        let cmd_pool = &self.cmd_pools[render_frame_idx];
        let cmd_buffer = &self.cmd_buffers[render_frame_idx];
        let render_texture = &self.resolution_dependent_resources.render_images[render_frame_idx];
        let render_texture_rtv =
            &self.resolution_dependent_resources.render_image_rtvs[render_frame_idx];
        let copy_texture = &self.resolution_dependent_resources.copy_images[render_frame_idx];

        cmd_pool.reset_command_pool().unwrap();
        cmd_buffer.begin().unwrap();

        //
        // RenderPass
        //
        render_surface.transition_to(cmd_buffer, ResourceState::SHADER_RESOURCE);

        cmd_buffer
            .cmd_resource_barrier(
                &[],
                &[TextureBarrier::<DefaultApi>::state_transition(
                    render_texture,
                    ResourceState::COPY_SRC,
                    ResourceState::RENDER_TARGET,
                )],
            )
            .unwrap();

        cmd_buffer
            .cmd_begin_render_pass(
                &[ColorRenderTargetBinding {
                    texture_view: render_texture_rtv,
                    load_op: LoadOp::DontCare,
                    store_op: StoreOp::Store,
                    clear_value: ColorClearValue::default(),
                }],
                None,
            )
            .unwrap();

        cmd_buffer.cmd_bind_pipeline(&self.pipeline).unwrap();

        self.descriptor_set_arrays[0]
            .update_descriptor_set(&[
                DescriptorUpdate {
                    array_index: render_frame_idx as u32,
                    descriptor_key: DescriptorKey::Name("hdr_sampler"),
                    elements: DescriptorElements {
                        samplers: Some(&[&self.bilinear_sampler]),
                        ..DescriptorElements::default()
                    },
                    ..DescriptorUpdate::default()
                },
                DescriptorUpdate {
                    array_index: render_frame_idx as u32,
                    descriptor_key: DescriptorKey::Name("hdr_image"),
                    elements: DescriptorElements {
                        texture_views: Some(&[render_surface.shader_resource_view()]),
                        ..DescriptorElements::default()
                    },
                    ..DescriptorUpdate::default()
                },
            ])
            .unwrap();

        cmd_buffer
            .cmd_bind_descriptor_set(
                &self.root_signature,
                &self.descriptor_set_arrays[0],
                (render_frame_idx) as _,
            )
            .unwrap();

        cmd_buffer.cmd_draw(3, 0).unwrap();

        cmd_buffer.cmd_end_render_pass().unwrap();

        cmd_buffer
            .cmd_resource_barrier(
                &[],
                &[TextureBarrier::<DefaultApi>::state_transition(
                    render_texture,
                    ResourceState::RENDER_TARGET,
                    ResourceState::COPY_SRC,
                )],
            )
            .unwrap();

        //
        // Copy
        //

        cmd_buffer
            .cmd_resource_barrier(
                &[],
                &[TextureBarrier::<DefaultApi>::state_transition(
                    copy_texture,
                    ResourceState::COMMON,
                    ResourceState::COPY_DST,
                )],
            )
            .unwrap();

        let copy_extents = render_texture.definition().extents;
        assert_eq!(copy_texture.definition().extents, copy_extents);

        cmd_buffer
            .cmd_copy_image(
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
                },
            )
            .unwrap();

        cmd_buffer
            .cmd_resource_barrier(
                &[],
                &[TextureBarrier::<DefaultApi>::state_transition(
                    copy_texture,
                    ResourceState::COPY_DST,
                    ResourceState::COMMON,
                )],
            )
            .unwrap();
        cmd_buffer.end().unwrap();

        //
        // Present the image
        //

        graphics_queue
            .submit(&[cmd_buffer], &[wait_sem], &[], None)
            .unwrap();

        graphics_queue.wait_for_queue_idle().unwrap();

        let sub_resource = copy_texture.map_texture().unwrap();
        copy_fn(sub_resource.data, sub_resource.row_pitch as usize);
        copy_texture.unmap_texture().unwrap();
        Ok(())
    }
}
