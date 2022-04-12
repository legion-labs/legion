use lgn_app::{App, Plugin};
use lgn_ecs::prelude::*;
use lgn_ecs::world::World;

use crate::labels::CommandBufferLabel;
use crate::labels::RenderStage;

use super::RenderThread;

pub struct RendererThreadPlugin {}

impl Plugin for RendererThreadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RenderThread::new());

        app.add_system_to_stage(
            RenderStage::Prepare,
            wait_for_prev_frame.before(CommandBufferLabel::RenderThread),
        );

        app.add_system_to_stage(
            RenderStage::Render,
            render_update
                .exclusive_system()
                .label(CommandBufferLabel::RenderThread),
        );
    }
}

fn visibility(_world: &mut World) {}

fn extract(_world: &mut World) {}

#[allow(clippy::needless_pass_by_value)]
fn wait_for_prev_frame(mut render_thread: ResMut<'_, RenderThread>) {
    render_thread.wait_for_previous_render_frame();
}

#[allow(clippy::needless_pass_by_value)]
fn render_update(world: &mut World) {
    visibility(world);

    extract(world);

    let mut render_thread = world.resource_mut::<RenderThread>();
    render_thread.kickoff_render_frame();
}
