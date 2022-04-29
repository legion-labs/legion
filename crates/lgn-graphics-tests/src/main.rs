use lgn_graphics_tests::render_passes::{
    AlphaBlendedLayerPass, DepthLayerPass, GpuCullingPass, LightingPass, OpaqueLayerPass,
    PostProcessPass, SSAOPass, UiPass,
};
use lgn_graphics_tests::render_script::{
    Config, Format, RenderGraphExecuteContext, RenderScript, RenderTarget, RenderTargetDesc,
    RenderView,
};

fn test_render_graph() {
    let gpu_culling_pass = GpuCullingPass {};
    let depth_layer_pass = DepthLayerPass {};
    let opaque_layer_pass = OpaqueLayerPass {};
    let ssao_pass = SSAOPass {};
    let alphablended_layer_pass = AlphaBlendedLayerPass {};
    let postprocess_pass = PostProcessPass {};
    let lighting_pass = LightingPass {};
    let ui_pass = UiPass {};

    let view_desc = RenderTargetDesc {
        name: "ViewBuffer".to_string(),
        width: 1920,
        height: 1080,
        depth: 1,
        array_size: 1,
        format: Format::R8G8B8A8_UNORM,
    };
    let view_target = RenderTarget {
        id: 0,
        desc: view_desc,
    };
    let view = RenderView {
        target: view_target,
    };

    let depth_desc = RenderTargetDesc {
        name: "PrevDepthBuffer".to_string(),
        width: 1920,
        height: 1080,
        depth: 1,
        array_size: 1,
        format: Format::D24_UNORM_S8_UINT,
    };
    let prev_depth = RenderTarget {
        id: 0,
        desc: depth_desc,
    };

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
            let execute_context: RenderGraphExecuteContext = RenderGraphExecuteContext {
                name: "Blah".to_string(),
            };

            // Print out the render graph
            println!("{}", render_graph);
            println!("\n\n");

            // Execute it (which currently just prints the passes in each pass execute function)
            render_graph.execute(&execute_context);
        }
        Err(error) => {
            println!("{}", error.msg);
        }
    }
}

fn main() {
    test_render_graph();
}
