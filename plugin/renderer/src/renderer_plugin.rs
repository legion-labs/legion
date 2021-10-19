use graphics_renderer::Renderer;
use legion_app::{CoreStage, Plugin};
use legion_ecs::{prelude::*, system::IntoSystem};
use log::debug;

use crate::{RenderOutput, RenderOutputDestroyed};

use super::labels::*;

#[derive(Default)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut legion_app::App) {
        let renderer = Renderer::new(1024, 1024);
        app.insert_resource(renderer);
        app.add_event::<RenderOutputDestroyed>();
        app.add_system_set( SystemSet::new()            
                .with_system(on_render_output_added.system().chain(render.system())
            ).label(RendererSystemLabel::Main)            
        ); 
        app.add_system_to_stage(CoreStage::PostUpdate, on_render_output_removed.system());
    }
}

fn on_render_output_added(    
    query_added: Query< (Entity,&RenderOutput), Added<RenderOutput> > ,
    query_changed: Query< (Entity,&RenderOutput), Changed<RenderOutput> > ,
) {
    
    for q in query_added.iter() {
        debug!( "added {:?}", q.0  );
    }

    for q in query_changed.iter() {
        debug!( "change {:?}", q.0  );
    }
}

fn on_render_output_removed(removed_components : RemovedComponents<RenderOutput>
) {
    for entity in removed_components.iter() {
        debug!( "removed {:?}", entity  );
    }
}

fn render(mut renderer: ResMut<Renderer>) {

    // let create_output_event_reader = ManualEventReader::<CreateOutput>::default();    
    
    // create_output_event_reader.iter(events)

    renderer.render();
}
