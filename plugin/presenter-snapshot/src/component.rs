#![allow(clippy::pedantic)]

use graphics_api::prelude::*;
use legion_ecs::prelude::Component;
use legion_presenter::offscreen_helper::{self, Resolution};
use legion_renderer::{
    components::{RenderSurface, RenderSurfaceId},
    Renderer,
};

#[derive(Component)]
pub struct PresenterSnapshot {
    frame_idx: i32,
    frame_target: i32,
    render_surface_id: RenderSurfaceId,
    offscreen_helper: offscreen_helper::OffscreenHelper,
}

impl std::fmt::Debug for PresenterSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PresenterSnapshot").finish()
    }
}

impl PresenterSnapshot {
    pub fn new(
        renderer: &Renderer,
        render_surface_id: RenderSurfaceId,
        resolution: Resolution,
    ) -> anyhow::Result<Self> {
        let device_context = renderer.device_context();
        let graphics_queue = renderer.graphics_queue();
        let offscreen_helper =
            offscreen_helper::OffscreenHelper::new(device_context, graphics_queue, resolution)?;

        Ok(Self {
            frame_idx: 0,
            frame_target: 0,
            render_surface_id,
            offscreen_helper,
        })
    }

    pub(crate) fn present(
        &mut self,
        graphics_queue: &<DefaultApi as GfxApi>::Queue,
        transient_descriptor_heap: &<DefaultApi as GfxApi>::DescriptorHeap,
        wait_sem: &<DefaultApi as GfxApi>::Semaphore,
        render_surface: &mut RenderSurface,
    ) -> anyhow::Result<bool> {
        //
        // Render
        //
        let snapshot_frame = self.frame_idx == self.frame_target;
        self.offscreen_helper.present(
            graphics_queue,
            transient_descriptor_heap,
            wait_sem,
            render_surface,
            |rgba: &[u8], row_pitch: usize| {
                // write frame to file
                if snapshot_frame {
                    let file =
                        std::fs::File::create(format!("presenter_snapshot_{}.png", self.frame_idx))
                            .unwrap();
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
