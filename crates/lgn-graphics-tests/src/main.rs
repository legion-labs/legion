use lgn_graphics_api::{
    ApiDef, CommandPoolDef, DeviceContext, Extents3D, Format, GfxApi, MemoryUsage, QueueType,
    ResourceFlags, ResourceUsage, TextureDef, TextureTiling,
};
use lgn_graphics_tests::render_passes::{
    AlphaBlendedLayerPass, DepthLayerPass, GpuCullingPass, LightingPass, OpaqueLayerPass,
    PostProcessPass, SSAOPass, UiPass,
};
use lgn_graphics_tests::render_script::{Config, RenderScript, RenderView};

fn test_render_graph(device_context: &DeviceContext) {
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
        usage_flags: ResourceUsage::AS_RENDER_TARGET | ResourceUsage::AS_SHADER_RESOURCE,
        resource_flags: ResourceFlags::empty(),
        mem_usage: MemoryUsage::GpuOnly,
        tiling: TextureTiling::Optimal,
    };
    let view_target = device_context.create_texture(&view_desc);
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
    let prev_depth = device_context.create_texture(&depth_desc);

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

            // TODO: Questions:
            // * Management of textures: pool for now, aliasing later
            // * Management of command buffers: one command buffer per pass for now
            // * Multithreaded execution: none for now

            let queue = device_context.create_queue(QueueType::Graphics).unwrap();
            let command_pool = queue
                .create_command_pool(&CommandPoolDef { transient: true })
                .unwrap();

            let mut context = render_graph.compile();

            println!("\n\n");

            // Execute it
            for i in 0..30 {
                println!(
                    "*****************************************************************************"
                );
                println!("Frame {}", i);
                render_graph.execute(&mut context, device_context, &command_pool);
            }
        }
        Err(error) => {
            println!("{}", error);
        }
    }
}

fn main() {
    let api = unsafe { GfxApi::new(&ApiDef::default()).unwrap() };

    test_render_graph(api.device_context());
}
