#![allow(clippy::pedantic)]

use graphics_api::prelude::*;
use legion_ecs::prelude::Component;
use legion_presenter::offscreen_helper::{self, Resolution};
use legion_renderer::{components::RenderSurface, Renderer};

#[derive(Component)]
pub struct PresenterSnapshot {
    frame_id: i32,
    frame_target: i32,
    offscreen_helper: offscreen_helper::OffscreenHelper,
}

impl std::fmt::Debug for PresenterSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PresenterSnapshot").finish()
    }
}

impl PresenterSnapshot {
    pub fn new(renderer: &Renderer, resolution: Resolution) -> anyhow::Result<Self> {
        let device_context = renderer.device_context();
        let graphics_queue = renderer.graphics_queue();
        let offscreen_helper =
            offscreen_helper::OffscreenHelper::new(device_context, graphics_queue, resolution)?;

        Ok(Self {
            frame_id: 0,
            frame_target: 0,
            offscreen_helper,
        })
    }

    pub(crate) fn _resize(
        &mut self,
        renderer: &Renderer,
        resolution: Resolution,
    ) -> anyhow::Result<()> {
        let device_context = renderer.device_context();
        self.offscreen_helper.resize(device_context, resolution)?;
        Ok(())
    }

    pub(crate) fn present(
        &mut self,
        graphics_queue: &<DefaultApi as GfxApi>::Queue,
        transient_descriptor_heap: &<DefaultApi as GfxApi>::DescriptorHeap,
        wait_sem: &<DefaultApi as GfxApi>::Semaphore,
        render_surface: &mut RenderSurface,
    ) -> anyhow::Result<()> {
        //
        // Render
        //
        self.offscreen_helper.present(
            graphics_queue,
            transient_descriptor_heap,
            wait_sem,
            render_surface,
            |_rgba: &[u8], _row_pitch: usize| {
                // write frame to file
                if self.frame_id == self.frame_target {
                    //let mut file =
                    //    std::fs::File::create(format!("presenter_snapshot_{}.png", self.frame_id))
                    //        .unwrap();
                }
            },
        )?;

        self.frame_id += 1;
        Ok(())
    }
}
