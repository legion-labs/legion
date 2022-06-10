use lgn_graphics_api::{BufferViewDef, ResourceUsage};
use lgn_math::Vec2;

use crate::cgen::cgen_type;
use crate::core::{
    DebugStuff, PrepareRenderContext, RenderFeatures, RenderLayers, RenderObjects,
    VisibilityContext,
};
use crate::gpu_renderer::GpuInstanceManager;
use crate::lighting::LightingManager;
use crate::resources::{PersistentDescriptorSetManager, SamplerManager};
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
            render_resources
                .get_mut::<SamplerManager>()
                .upload(persistent_descriptor_set_manager);
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

            for viewport in render_surface.viewports_mut() {
                // TODO: #1997 From this point, we should be per RenderViewport, each viewport owning its own top level camera and properties (grid, gizmos etc...).

                //
                // Visibility
                //

                let render_camera = viewport.camera();

                let visibility_context = VisibilityContext {
                    herd: render_context.herd,
                    bump: render_context.bump,
                    render_camera,
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
                            viewport.extents().width as f32,
                            viewport.extents().height as f32,
                        );
                    }

                    let cursor_pos = render_context.picking_manager.current_cursor_pos();

                    let view_data = render_camera.tmp_build_view_data(
                        viewport.extents().width as f32,
                        viewport.extents().height as f32,
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

                viewport.clear_hzb_if_needed(cmd_buffer);

                cmd_buffer.end();

                render_context
                    .graphics_queue
                    .queue_mut()
                    .submit(&[cmd_buffer], &[], &[], None);

                render_context
                    .transient_commandbuffer_allocator
                    .release(cmd_buffer_handle);

                let view = RenderView {
                    target: viewport.view_target(),
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
                    hzb: [viewport.hzb()[0], viewport.hzb()[1]],
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

            let mut cmd_buffer_handle = render_context.transient_commandbuffer_allocator.acquire();
            let cmd_buffer = cmd_buffer_handle.as_mut();

            cmd_buffer.begin();

            cmd_buffer.end();

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
