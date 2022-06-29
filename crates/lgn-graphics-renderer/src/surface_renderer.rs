use lgn_graphics_api::{BufferViewDef, ResourceUsage};
use lgn_math::Vec2;

use crate::cgen::cgen_type;
use crate::core::{
    DebugStuff, PrepareRenderContext, RenderCamera, RenderFeatures, RenderLayers, RenderObjects,
    RenderViewport, RenderViewportRendererData, VisibilityContext,
};
use crate::gpu_renderer::GpuInstanceManager;
use crate::lighting::LightingManager;
use crate::resources::PersistentDescriptorSetManager;
use crate::script::render_passes::{
    AlphaBlendedLayerPass, DebugPass, EguiPass, GpuCullingPass, LightingPass, OpaqueLayerPass,
    PickingPass, PostProcessPass, SSAOPass, UiPass,
};
use crate::script::{Config, RenderScript, RenderView};
use crate::{cgen, RenderContext};
use crate::{components::RenderSurfaces, core::RenderResources};

pub(crate) struct SurfaceRenderer;

impl SurfaceRenderer {
    pub(crate) fn render_surfaces<'a>(
        frame_idx: u64,
        render_surfaces: &mut RenderSurfaces,
        render_resources: &'a RenderResources,
        mut render_context: RenderContext<'a>,
        persistent_descriptor_set_manager: &'a mut PersistentDescriptorSetManager,
        render_layers: &'a RenderLayers,
        features: &'a RenderFeatures,
    ) {
        // Persistent descriptor set
        {
            let descriptor_set = persistent_descriptor_set_manager.descriptor_set();
            render_context
                .set_persistent_descriptor_set(descriptor_set.layout(), *descriptor_set.handle());
        }

        // Frame descriptor set
        {
            let mut frame_descriptor_set = cgen::descriptor_set::FrameDescriptorSet::default();
            let render_objects = render_resources.get::<RenderObjects>();

            render_resources.get::<LightingManager>().per_frame_render(
                &render_objects,
                render_context.transient_buffer_allocator,
                &mut frame_descriptor_set,
            );

            let static_buffer_ro_view = render_context.static_buffer.read_only_view();
            frame_descriptor_set.set_static_buffer(static_buffer_ro_view);

            let instance_manager = render_resources.get::<GpuInstanceManager>();
            let va_table_address_buffer = instance_manager.structured_buffer_view();
            frame_descriptor_set.set_va_table_address_buffer(va_table_address_buffer);

            let frame_descriptor_set_handle = render_context.write_descriptor_set(
                cgen::descriptor_set::FrameDescriptorSet::descriptor_set_layout(),
                frame_descriptor_set.descriptor_refs(),
            );

            render_context.set_frame_descriptor_set(
                cgen::descriptor_set::FrameDescriptorSet::descriptor_set_layout(),
                frame_descriptor_set_handle,
            );
        }

        // For each surface/view, we have to execute the render graph
        for render_surface in render_surfaces.iter_mut() {
            let picking_renderpass = render_surface.picking_renderpass();

            let viewports = render_surface.viewports();

            let render_objects = render_resources.get::<RenderObjects>();
            let viewport_primary_table = render_objects.primary_table::<RenderViewport>();
            let mut viewport_secondary_table =
                render_objects.secondary_table_mut::<RenderViewportRendererData>();
            let camera_primary_table = render_objects.primary_table::<RenderCamera>();

            for viewport in viewports {
                let render_viewport_id = viewport.render_object_id();
                if render_viewport_id.is_none() {
                    continue;
                }
                let render_viewport_id = render_viewport_id.unwrap();

                let render_viewport =
                    viewport_primary_table.get::<RenderViewport>(render_viewport_id);
                let render_viewport_renderer_data =
                    viewport_secondary_table
                        .get_mut::<RenderViewportRendererData>(render_viewport_id);

                //
                // Visibility
                //

                let render_camera_id = render_viewport.camera_id();
                if render_camera_id.is_none() {
                    continue;
                }
                let render_camera_id = render_camera_id.unwrap();
                let render_camera = camera_primary_table.get::<RenderCamera>(render_camera_id);

                let visibility_context = VisibilityContext {
                    herd: render_context.herd,
                    bump: render_context.bump,
                    render_camera: render_camera_id,
                    render_layers,
                };

                let visibility_set = visibility_context.execute();

                //
                // Update
                //

                // ==== TODO ====

                //
                // PrepareRender
                //

                let prepare_render_context = PrepareRenderContext {
                    herd: render_context.herd,
                    bump: render_context.bump,
                    visibility_set,
                    features,
                };

                let render_list_set = prepare_render_context.execute();

                // View descriptor set
                {
                    let mut screen_rect = render_context.picking_manager.screen_rect();
                    if screen_rect.x == 0.0 || screen_rect.y == 0.0 {
                        screen_rect = Vec2::new(
                            render_viewport.extents().width as f32,
                            render_viewport.extents().height as f32,
                        );
                    }

                    let cursor_pos = render_context.picking_manager.current_cursor_pos();

                    let view_data = render_camera.tmp_build_view_data(
                        render_viewport.extents().width as f32,
                        render_viewport.extents().height as f32,
                        screen_rect.x,
                        screen_rect.y,
                        cursor_pos.x,
                        cursor_pos.y,
                    );

                    let sub_allocation = render_context
                        .transient_buffer_allocator
                        .copy_data(&view_data, ResourceUsage::AS_CONST_BUFFER);

                    let const_buffer_view =
                        sub_allocation.to_buffer_view(BufferViewDef::as_const_buffer_typed::<
                            cgen_type::ViewData,
                        >());

                    let mut view_descriptor_set =
                        cgen::descriptor_set::ViewDescriptorSet::default();
                    view_descriptor_set.set_view_data(const_buffer_view);

                    let view_descriptor_set_handle = render_context.write_descriptor_set(
                        cgen::descriptor_set::ViewDescriptorSet::descriptor_set_layout(),
                        view_descriptor_set.descriptor_refs(),
                    );

                    render_context.set_view_descriptor_set(
                        cgen::descriptor_set::ViewDescriptorSet::descriptor_set_layout(),
                        view_descriptor_set_handle,
                    );
                }

                let mut cmd_buffer_handle =
                    render_context.transient_commandbuffer_allocator.acquire();
                let cmd_buffer = cmd_buffer_handle.as_mut();

                cmd_buffer.begin();

                render_viewport_renderer_data.clear_hzb_if_needed(cmd_buffer);

                cmd_buffer.end();

                render_context
                    .graphics_queue
                    .queue_mut()
                    .submit(&[cmd_buffer], &[], &[], None);

                render_context
                    .transient_commandbuffer_allocator
                    .release(cmd_buffer_handle);

                let view = RenderView {
                    target: render_viewport_renderer_data.view_target(),
                };

                let gpu_culling_pass = GpuCullingPass;
                let picking_pass = PickingPass;
                let opaque_layer_pass = OpaqueLayerPass;
                let ssao_pass = SSAOPass;
                let alphablended_layer_pass = AlphaBlendedLayerPass;
                let debug_pass = DebugPass;
                let postprocess_pass = PostProcessPass;
                let lighting_pass = LightingPass;
                let ui_pass = UiPass;
                let egui_pass = EguiPass;

                let mut render_script = RenderScript {
                    gpu_culling_pass,
                    picking_pass,
                    opaque_layer_pass,
                    ssao_pass,
                    alphablended_layer_pass,
                    debug_pass,
                    postprocess_pass,
                    lighting_pass,
                    ui_pass,
                    egui_pass,
                    hzb: [
                        render_viewport_renderer_data.hzb()[0],
                        render_viewport_renderer_data.hzb()[1],
                    ],
                };

                let config = Config {
                    frame_idx,
                    ..Config::default()
                };

                match render_script.build_render_graph(
                    &view,
                    &config,
                    render_resources,
                    render_context.pipeline_manager,
                    render_context.device_context,
                ) {
                    Ok(render_graph) => {
                        let mut render_graph_context = render_graph.compile();

                        let debug_stuff = DebugStuff {
                            picking_renderpass: &picking_renderpass,
                            render_viewport,
                            render_camera,
                            egui: render_context.egui,
                        };

                        render_graph.execute(
                            &mut render_graph_context,
                            render_list_set,
                            render_resources,
                            &mut render_context,
                            &debug_stuff,
                        );
                    }
                    Err(error) => {
                        println!("{}", error);
                    }
                }

                render_list_set.consume();
            }

            //---------------- composite viewports
            let mut cmd_buffer_handle = render_context.transient_commandbuffer_allocator.acquire();
            let cmd_buffer = cmd_buffer_handle.as_mut();

            cmd_buffer.begin();

            let mut render_viewports = vec![];
            let mut render_viewports_private_data = vec![];

            for viewport in viewports {
                if let Some(render_object_id) = viewport.render_object_id() {
                    let render_viewport =
                        viewport_primary_table.get::<RenderViewport>(render_object_id);
                    let render_viewport_private_data =
                        viewport_secondary_table
                            .get::<RenderViewportRendererData>(render_object_id);

                    render_viewports.push(render_viewport);
                    render_viewports_private_data.push(render_viewport_private_data);
                }
            }

            render_surface.composite_viewports(
                &render_viewports,
                &render_viewports_private_data,
                cmd_buffer,
            );

            cmd_buffer.end();
            //---------------- composite viewports

            // queue
            let present_semaphore = render_surface.acquire();
            {
                render_context.graphics_queue.queue_mut().submit(
                    &[cmd_buffer],
                    &[],
                    &[present_semaphore],
                    None,
                );

                render_surface.present(&mut render_context);
            }

            render_context
                .transient_commandbuffer_allocator
                .release(cmd_buffer_handle);
        }
    }
}
