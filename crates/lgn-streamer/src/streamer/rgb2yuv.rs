use lgn_graphics_api::prelude::*;
use lgn_graphics_cgen_runtime::CGenShaderKey;
use lgn_graphics_renderer::{
    components::RenderSurface,
    resources::{PipelineDef, PipelineHandle, PipelineManager},
    RenderContext,
};
use lgn_tracing::span_fn;

use crate::cgen;

use super::Resolution;

struct ResolutionDependentResources {
    resolution: Resolution,
    yuv_images: Vec<(Texture, Texture, Texture)>,
    yuv_image_uavs: Vec<(TextureView, TextureView, TextureView)>,
    copy_yuv_images: Vec<Texture>,
}

impl ResolutionDependentResources {
    fn new(
        device_context: &DeviceContext,
        render_frame_count: u32,
        resolution: Resolution,
    ) -> Self {
        let mut yuv_images = Vec::with_capacity(render_frame_count as usize);
        let mut yuv_image_uavs = Vec::with_capacity(render_frame_count as usize);
        let mut copy_yuv_images = Vec::with_capacity(render_frame_count as usize);
        for _ in 0..render_frame_count {
            let y_image = device_context.create_texture(
                TextureDef {
                    extents: Extents3D {
                        width: resolution.width,
                        height: resolution.height,
                        depth: 1,
                    },
                    array_length: 1,
                    mip_count: 1,
                    format: Format::R8_UNORM,
                    memory_usage: MemoryUsage::GpuOnly,
                    usage_flags: ResourceUsage::AS_UNORDERED_ACCESS
                        | ResourceUsage::AS_TRANSFERABLE,
                    resource_flags: ResourceFlags::empty(),
                    tiling: TextureTiling::Optimal,
                },
                "Y",
            );
            let mut u_plane_def = *y_image.definition();
            u_plane_def.extents.width /= 2;
            u_plane_def.extents.height /= 2;
            let v_plane_def = u_plane_def;
            let u_image = device_context.create_texture(u_plane_def, "U");
            let v_image = device_context.create_texture(v_plane_def, "V");

            let yuv_plane_uav_def = TextureViewDef {
                gpu_view_type: GPUViewType::UnorderedAccess,
                view_dimension: ViewDimension::_2D,
                first_mip: 0,
                mip_count: 1,
                plane_slice: PlaneSlice::Default,
                first_array_slice: 0,
                array_size: 1,
            };

            let y_image_uav = y_image.create_view(yuv_plane_uav_def);
            let u_image_uav = u_image.create_view(yuv_plane_uav_def);
            let v_image_uav = v_image.create_view(yuv_plane_uav_def);

            let copy_yuv_image = device_context.create_texture(
                TextureDef {
                    extents: Extents3D {
                        width: resolution.width,
                        height: resolution.height,
                        depth: 1,
                    },
                    array_length: 1,
                    mip_count: 1,
                    format: Format::G8_B8_R8_3PLANE_420_UNORM,
                    memory_usage: MemoryUsage::GpuToCpu,
                    usage_flags: ResourceUsage::AS_TRANSFERABLE,
                    resource_flags: ResourceFlags::empty(),
                    tiling: TextureTiling::Linear,
                },
                "Copy",
            );

            yuv_images.push((y_image, u_image, v_image));
            yuv_image_uavs.push((y_image_uav, u_image_uav, v_image_uav));
            copy_yuv_images.push(copy_yuv_image);
        }

        Self {
            resolution,
            yuv_images,
            yuv_image_uavs,
            copy_yuv_images,
        }
    }
}

pub struct RgbToYuvConverter {
    render_frame_count: u32,
    resolution_dependent_resources: ResolutionDependentResources,
    pipeline_handle: PipelineHandle,
}

impl RgbToYuvConverter {
    pub fn new(
        pipeline_manager: &PipelineManager,
        device_context: &DeviceContext,
        resolution: Resolution,
    ) -> Self {
        let root_signature = cgen::pipeline_layout::RGB2YUVPipelineLayout::root_signature();

        let shader = pipeline_manager
            .create_shader(
                cgen::CRATE_ID,
                CGenShaderKey::make(
                    cgen::shader::rgb2yuv_shader::ID,
                    cgen::shader::rgb2yuv_shader::NONE,
                ),
            )
            .unwrap();
        let pipeline_handle =
            pipeline_manager.register_pipeline(PipelineDef::Compute(ComputePipelineDef {
                shader,
                root_signature: root_signature.clone(),
            }));

        ////////////////////////////////////////////////////////////////////////////////

        let render_frame_count = 1u32;
        let resolution_dependent_resources =
            ResolutionDependentResources::new(device_context, render_frame_count, resolution);

        Self {
            render_frame_count: 1,
            resolution_dependent_resources,
            pipeline_handle,
        }
    }

    pub fn resize(&mut self, device_context: &DeviceContext, resolution: Resolution) -> bool {
        if resolution != self.resolution_dependent_resources.resolution {
            self.resolution_dependent_resources = ResolutionDependentResources::new(
                device_context,
                self.render_frame_count,
                resolution,
            );
            true
        } else {
            false
        }
    }

    #[span_fn]
    pub fn convert(
        &mut self,
        render_context: &mut RenderContext<'_>,
        render_surface: &mut RenderSurface,
        yuv: &mut [u8],
    ) {
        let render_frame_idx = 0;
        let mut cmd_buffer_handle = render_context.transient_commandbuffer_allocator.acquire();
        let cmd_buffer = cmd_buffer_handle.as_mut();

        cmd_buffer.begin();

        let view_target = render_surface.view_target();

        cmd_buffer.cmd_resource_barrier(&[], &[]);

        // TODO(jsg): Ideally we should not create this view each frame.
        let final_target_srv = view_target.create_view(TextureViewDef::as_shader_resource_view(
            view_target.definition(),
        ));

        {
            let yuv_images = &self.resolution_dependent_resources.yuv_images[render_frame_idx];
            cmd_buffer.cmd_resource_barrier(
                &[],
                &[
                    TextureBarrier::state_transition(
                        view_target,
                        ResourceState::RENDER_TARGET,
                        ResourceState::SHADER_RESOURCE,
                    ),
                    TextureBarrier::state_transition(
                        &yuv_images.0,
                        ResourceState::COPY_SRC,
                        ResourceState::UNORDERED_ACCESS,
                    ),
                    TextureBarrier::state_transition(
                        &yuv_images.1,
                        ResourceState::COPY_SRC,
                        ResourceState::UNORDERED_ACCESS,
                    ),
                    TextureBarrier::state_transition(
                        &yuv_images.2,
                        ResourceState::COPY_SRC,
                        ResourceState::UNORDERED_ACCESS,
                    ),
                ],
            );

            let pipeline = render_context
                .pipeline_manager
                .get_pipeline(self.pipeline_handle)
                .unwrap();
            cmd_buffer.cmd_bind_pipeline(pipeline);

            let yuv_images_views =
                &self.resolution_dependent_resources.yuv_image_uavs[render_frame_idx];

            let mut descriptor_set = cgen::descriptor_set::RGB2YUVDescriptorSet::default();
            descriptor_set.set_hdr_image(&final_target_srv);
            descriptor_set.set_y_image(&yuv_images_views.0);
            descriptor_set.set_u_image(&yuv_images_views.1);
            descriptor_set.set_v_image(&yuv_images_views.2);

            let descriptor_set_handle = render_context.write_descriptor_set(
                cgen::descriptor_set::RGB2YUVDescriptorSet::descriptor_set_layout(),
                descriptor_set.descriptor_refs(),
            );
            cmd_buffer.cmd_bind_descriptor_set_handle(
                cgen::descriptor_set::RGB2YUVDescriptorSet::descriptor_set_layout(),
                descriptor_set_handle,
            );

            cmd_buffer.cmd_dispatch(
                ((self.resolution_dependent_resources.resolution.width + 7) / 8) as u32,
                ((self.resolution_dependent_resources.resolution.height + 7) / 8) as u32,
                1,
            );
        }

        let copy_texture_yuv =
            &self.resolution_dependent_resources.copy_yuv_images[render_frame_idx];
        {
            let yuv_images = &self.resolution_dependent_resources.yuv_images[render_frame_idx];
            cmd_buffer.cmd_resource_barrier(
                &[],
                &[
                    TextureBarrier::state_transition(
                        view_target,
                        ResourceState::SHADER_RESOURCE,
                        ResourceState::RENDER_TARGET,
                    ),
                    TextureBarrier::state_transition(
                        &yuv_images.0,
                        ResourceState::UNORDERED_ACCESS,
                        ResourceState::COPY_SRC,
                    ),
                    TextureBarrier::state_transition(
                        &yuv_images.1,
                        ResourceState::UNORDERED_ACCESS,
                        ResourceState::COPY_SRC,
                    ),
                    TextureBarrier::state_transition(
                        &yuv_images.2,
                        ResourceState::UNORDERED_ACCESS,
                        ResourceState::COPY_SRC,
                    ),
                    TextureBarrier::state_transition(
                        copy_texture_yuv,
                        ResourceState::COMMON,
                        ResourceState::COPY_DST,
                    ),
                ],
            );

            let mut copy_extents = copy_texture_yuv.definition().extents;
            assert_eq!(copy_texture_yuv.definition().extents, copy_extents);
            cmd_buffer.cmd_copy_image(
                &yuv_images.0,
                copy_texture_yuv,
                &CmdCopyTextureParams {
                    src_state: ResourceState::COPY_SRC,
                    dst_state: ResourceState::COPY_DST,
                    src_offset: Offset3D { x: 0, y: 0, z: 0 },
                    dst_offset: Offset3D { x: 0, y: 0, z: 0 },
                    src_mip_level: 0,
                    dst_mip_level: 0,
                    src_array_slice: 0,
                    dst_array_slice: 0,
                    src_plane_slice: PlaneSlice::Default,
                    dst_plane_slice: PlaneSlice::Plane0,
                    extent: copy_extents,
                },
            );

            copy_extents.width /= 2;
            copy_extents.height /= 2;
            cmd_buffer.cmd_copy_image(
                &yuv_images.1,
                copy_texture_yuv,
                &CmdCopyTextureParams {
                    src_state: ResourceState::COPY_SRC,
                    dst_state: ResourceState::COPY_DST,
                    src_offset: Offset3D { x: 0, y: 0, z: 0 },
                    dst_offset: Offset3D { x: 0, y: 0, z: 0 },
                    src_mip_level: 0,
                    dst_mip_level: 0,
                    src_array_slice: 0,
                    dst_array_slice: 0,
                    src_plane_slice: PlaneSlice::Default,
                    dst_plane_slice: PlaneSlice::Plane1,
                    extent: copy_extents,
                },
            );

            cmd_buffer.cmd_copy_image(
                &yuv_images.2,
                copy_texture_yuv,
                &CmdCopyTextureParams {
                    src_state: ResourceState::COPY_SRC,
                    dst_state: ResourceState::COPY_DST,
                    src_offset: Offset3D { x: 0, y: 0, z: 0 },
                    dst_offset: Offset3D { x: 0, y: 0, z: 0 },
                    src_mip_level: 0,
                    dst_mip_level: 0,
                    src_array_slice: 0,
                    dst_array_slice: 0,
                    src_plane_slice: PlaneSlice::Default,
                    dst_plane_slice: PlaneSlice::Plane2,
                    extent: copy_extents,
                },
            );

            cmd_buffer.cmd_resource_barrier(
                &[],
                &[TextureBarrier::state_transition(
                    copy_texture_yuv,
                    ResourceState::COPY_DST,
                    ResourceState::COMMON,
                )],
            );
        }

        cmd_buffer.end();

        //
        // Present the image
        //

        let wait_sem = render_surface.presenter_sem();
        let mut graphics_queue = render_context.graphics_queue.queue_mut();

        graphics_queue.submit(&[cmd_buffer], &[wait_sem], &[], None);

        graphics_queue.wait_for_queue_idle();

        let (mut width, mut height) = (
            copy_texture_yuv.definition().extents.width as usize,
            copy_texture_yuv.definition().extents.height as usize,
        );

        let sub_resource = copy_texture_yuv.map_texture(PlaneSlice::Plane0).unwrap();
        let mut amount_copied = 0;
        for y in 0..height {
            yuv[amount_copied..(y + 1) * width].copy_from_slice(
                &sub_resource.data[y * sub_resource.row_pitch as usize
                    ..(y * sub_resource.row_pitch as usize) + width],
            );
            amount_copied += width;
        }
        copy_texture_yuv.unmap_texture();

        let sub_resource = copy_texture_yuv.map_texture(PlaneSlice::Plane1).unwrap();
        width /= 2;
        height /= 2;
        let start = amount_copied;
        for y in 0..height {
            yuv[amount_copied..start + (y + 1) * width].copy_from_slice(
                &sub_resource.data[y * sub_resource.row_pitch as usize
                    ..(y * sub_resource.row_pitch as usize) + width],
            );
            amount_copied += width;
        }
        copy_texture_yuv.unmap_texture();

        let sub_resource = copy_texture_yuv.map_texture(PlaneSlice::Plane2).unwrap();
        let start = amount_copied;
        for y in 0..height {
            yuv[amount_copied..start + (y + 1) * width].copy_from_slice(
                &sub_resource.data[y * sub_resource.row_pitch as usize
                    ..(y * sub_resource.row_pitch as usize) + width],
            );
            amount_copied += width;
        }
        copy_texture_yuv.unmap_texture();

        assert_eq!(amount_copied, yuv.len());
    }
}
