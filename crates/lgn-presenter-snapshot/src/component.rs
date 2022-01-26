#![allow(clippy::missing_errors_doc)]

use lgn_ecs::prelude::Component;
use lgn_renderer::{
    components::{Presenter, RenderSurface, RenderSurfaceExtents, RenderSurfaceId},
    RenderContext, Renderer,
};

use crate::OffscreenHelper;

#[derive(Component)]
pub struct PresenterSnapshot {
    snapshot_name: String,
    frame_idx: i32,
    frame_target: i32,
    render_surface_id: RenderSurfaceId,
    offscreen_helper: OffscreenHelper,
}

impl std::fmt::Debug for PresenterSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PresenterSnapshot").finish()
    }
}

impl PresenterSnapshot {
    pub fn new(
        snapshot_name: &str,
        frame_target: i32,
        renderer: &Renderer,
        render_surface_id: RenderSurfaceId,
        resolution: RenderSurfaceExtents,
    ) -> anyhow::Result<Self> {
        let device_context = renderer.device_context();
        let offscreen_helper =
            OffscreenHelper::new(&renderer.shader_compiler(), device_context, resolution)?;

        Ok(Self {
            snapshot_name: snapshot_name.to_string(),
            frame_idx: 0,
            frame_target,
            render_surface_id,
            offscreen_helper,
        })
    }

    pub(crate) fn present(
        &mut self,
        render_context: &RenderContext<'_>,
        render_surface: &mut RenderSurface,
    ) -> anyhow::Result<bool> {
        //
        // Render
        //
        let snapshot_frame = self.frame_idx == self.frame_target;
        self.offscreen_helper.present(
            render_context,
            render_surface,
            |rgba: &[u8], row_pitch: usize| {
                // write frame to file
                if snapshot_frame {
                    let file =
                        std::fs::File::create(format!("{}.png", self.snapshot_name)).unwrap();
                    let mut buf_writer = std::io::BufWriter::new(file);
                    let mut encoder = png::Encoder::new(
                        &mut buf_writer,
                        (row_pitch / 4) as u32,
                        (rgba.len() / row_pitch) as u32,
                    );
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    let mut writer = encoder.write_header().unwrap();
                    writer.write_image_data(rgba).unwrap(); // Save
                }
            },
        )?;

        self.frame_idx += 1;
        if snapshot_frame {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn render_surface_id(&self) -> RenderSurfaceId {
        self.render_surface_id
    }
}

impl Presenter for PresenterSnapshot {
    fn resize(
        &mut self,
        _renderer: &Renderer,
        _extents: lgn_renderer::components::RenderSurfaceExtents,
    ) {
        unreachable!();
    }

    fn present(&mut self, render_context: &RenderContext<'_>, render_surface: &mut RenderSurface) {
        self.present(render_context, render_surface).unwrap();
    }
}
