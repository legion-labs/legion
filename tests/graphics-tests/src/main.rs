use lgn_graphics_api::{
    Extents3D, Format, MemoryUsage, ResourceFlags, ResourceUsage, TextureDef, TextureTiling,
};
use lgn_graphics_renderer::core::render_passes::{
    AlphaBlendedLayerPass, DepthLayerPass, GpuCullingPass, LightingPass, OpaqueLayerPass,
    PostProcessPass, SSAOPass, UiPass,
};
use lgn_graphics_renderer::core::render_script::{Config, RenderScript, RenderView};
use lgn_graphics_renderer::resources::{DescriptorHeapManager, PipelineManager};
use lgn_graphics_renderer::{RenderContext, Renderer};

fn test_render_graph(renderer: &Renderer, render_context: &RenderContext<'_>) {
    let gpu_culling_pass = GpuCullingPass {};
    let depth_layer_pass = DepthLayerPass {};
    let opaque_layer_pass = OpaqueLayerPass {};
    let ssao_pass = SSAOPass {};
    let alphablended_layer_pass = AlphaBlendedLayerPass {};
    let postprocess_pass = PostProcessPass {};
    let lighting_pass = LightingPass {};
    let ui_pass = UiPass {};

    let view_desc = TextureDef {
        name: "ViewBuffer".to_string(),
        extents: Extents3D {
            width: 1920,
            height: 1080,
            depth: 1,
        },
        array_length: 1,
        mip_count: 1,
        format: Format::R8G8B8A8_UNORM,
        usage_flags: ResourceUsage::AS_RENDER_TARGET
            | ResourceUsage::AS_SHADER_RESOURCE
            | ResourceUsage::AS_TRANSFERABLE,
        resource_flags: ResourceFlags::empty(),
        mem_usage: MemoryUsage::GpuOnly,
        tiling: TextureTiling::Optimal,
    };
    let view_target = renderer.device_context().create_texture(&view_desc);
    let view = RenderView {
        target: view_target,
    };

    let depth_desc = TextureDef {
        name: "PrefDepthBuffer".to_string(),
        extents: view.target.definition().extents,
        array_length: 1,
        mip_count: 1,
        format: Format::D24_UNORM_S8_UINT,
        usage_flags: ResourceUsage::AS_DEPTH_STENCIL | ResourceUsage::AS_SHADER_RESOURCE,
        resource_flags: ResourceFlags::empty(),
        mem_usage: MemoryUsage::GpuOnly,
        tiling: TextureTiling::Optimal,
    };
    let prev_depth = renderer.device_context().create_texture(&depth_desc);

    let mut render_script = RenderScript {
        gpu_culling_pass,
        depth_layer_pass,
        opaque_layer_pass,
        ssao_pass,
        alphablended_layer_pass,
        postprocess_pass,
        lighting_pass,
        ui_pass,
        prev_depth,
    };

    let config = Config::default();

    match render_script.build_render_graph(&view, &config) {
        Ok(render_graph) => {
            // Print out the render graph
            println!("{}", render_graph);
            println!("\n\n");

            let mut context = render_graph.compile();

            println!("\n\n");

            // Execute it
            for i in 0..30 {
                println!(
                    "*****************************************************************************"
                );
                println!("Frame {}", i);
                let mut command_buffer = render_context.alloc_command_buffer();
                render_graph.execute(&mut context, renderer.device_context(), &mut command_buffer);
            }
        }
        Err(error) => {
            println!("{}", error);
        }
    }
}

fn main() {
    const NUM_RENDER_FRAMES: usize = 2;
    let renderer = Renderer::new(NUM_RENDER_FRAMES);

    let descriptor_heap_manager =
        DescriptorHeapManager::new(NUM_RENDER_FRAMES, renderer.device_context());

    let pipeline_manager = PipelineManager::new(renderer.device_context());

    let render_context = RenderContext::new(&renderer, &descriptor_heap_manager, &pipeline_manager);

    test_render_graph(&renderer, &render_context);
}
