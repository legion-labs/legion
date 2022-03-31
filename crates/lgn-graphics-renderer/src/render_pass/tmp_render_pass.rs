#![allow(unsafe_code)]

use lgn_graphics_api::{
    ColorClearValue, ColorRenderTargetBinding, DepthStencilClearValue,
    DepthStencilRenderTargetBinding, LoadOp, ResourceState, StoreOp,
};
use lgn_graphics_data::Color;
use lgn_tracing::span_fn;

use crate::{
    components::RenderSurface,
    gpu_renderer::{DefaultLayers, MeshRenderer},
    hl_gfx_api::HLCommandBuffer,
    RenderContext,
};

pub struct TmpRenderPass {}

impl TmpRenderPass {
    #[span_fn]
    pub(crate) fn render(
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        render_surface: &mut RenderSurface,
        mesh_renderer: &MeshRenderer,
    ) {
        render_surface
            .hdr_rt_mut()
            .transition_to(cmd_buffer, ResourceState::RENDER_TARGET);

        cmd_buffer.begin_render_pass(
            &[ColorRenderTargetBinding {
                texture_view: render_surface.hdr_rt().rtv(),
                load_op: LoadOp::Clear,
                store_op: StoreOp::Store,
                clear_value: ColorClearValue(Color::new(180, 180, 180, 255).as_linear().into()),
            }],
            &Some(DepthStencilRenderTargetBinding {
                texture_view: render_surface.depth_rt().rtv(),
                depth_load_op: LoadOp::Load,
                stencil_load_op: LoadOp::DontCare,
                depth_store_op: StoreOp::Store,
                stencil_store_op: StoreOp::DontCare,
                clear_value: DepthStencilClearValue {
                    depth: 1.0,
                    stencil: 0,
                },
            }),
        );

        mesh_renderer.draw(render_context, cmd_buffer, DefaultLayers::Opaque as usize);

        cmd_buffer.end_render_pass();
    }
}
