use graphics_api::{DeviceContext, GfxApi};
use legion_app::{Plugin};
use legion_ecs::{prelude::*, system::IntoSystem};

use crate::{Renderer, GPUResourceFactory, components::RenderSurface};

use super::labels::*;

#[derive(Default)]
pub struct RendererPlugin;

// struct RenderSurfaces {
//     render_surfaces: HashMap<WindowId, RenderSurface>
// }

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut legion_app::App) {
        let renderer = Renderer::new(1024, 1024);
        let gpu_resource_factory = GPUResourceFactory::new(renderer.api().device_context().clone());
        app.insert_resource(gpu_resource_factory);
        app.insert_resource(renderer);
        // app.insert_resource(RenderSurfaces);
        app.add_system_set( SystemSet::new()            
                .with_system(
                    // on_render_output_added.system().chain(
                        // on_render_output_changed.system().chain(
                            render.system()
                        )
                    //)
                //)
            .label(RendererSystemLabel::Main)            
        ); 
        // app.add_system_to_stage(CoreStage::PostUpdate, on_render_output_removed.system());
    }
}

// fn on_render_output_added(    
//     mut commands: Commands,    
//     query_added: Query<(Entity, &RenderOutput), Added<RenderOutput>> 
// ) {    
//     for (entity, _render_output) in query_added.iter() {                        
//         commands.entity(entity).insert(RenderSurface);
//     }
// }

// fn on_render_output_changed(            
//     mut query_changed: Query<(Entity, &RenderOutput, &mut RenderSurface), Changed<RenderOutput>> 
// ) {    
//     for (_, _, mut render_surface) in query_changed.iter_mut() {        
//         let _render_surface = &mut *render_surface;        
//     }
// }

// fn on_render_output_removed(    
//     removed_components : RemovedComponents<RenderSurface>
// ) {
//     for entity in removed_components.iter() {
//         debug!( "removed {:?}", entity  );
//     }
// }

fn render(
    mut renderer: ResMut<Renderer>,
    // outputs: Query<(Entity, &RenderOutput, &RenderSurface)> 
    outputs: Query<(Entity, &RenderSurface)> 
) {

    renderer.api().device_context().free_gpu_memory();

    for (_,_render_surface) in outputs.iter() {

        dbg!( "toto" );
        // render_surface.render();

    }

    // let create_output_event_reader = ManualEventReader::<CreateOutput>::default();    
    
    // create_output_event_reader.iter(events)

    renderer.render();
}
