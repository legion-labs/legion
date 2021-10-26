use graphics_api::{CommandBuffer, DefaultApi, ResourceState, TextureBarrier};
use legion_app::{Plugin};
use legion_ecs::{prelude::*, system::IntoSystem};

use crate::{Renderer, components::RenderSurface};

use super::labels::*;

#[derive(Default)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut legion_app::App) {        
        
        let renderer = Renderer::new();        

        app.insert_resource(renderer);                
        app.add_system_set( SystemSet::new()            
                .with_system(
                    render.system()
                )
            .label(RendererSystemLabel::Main)            
        ); 
    }
}

fn render(    
    mut renderer: ResMut<Renderer>,
    outputs: Query<(Entity, &RenderSurface)> 
) { 
    renderer.begin_frame();    

    let cmd_buffer = renderer.get_cmd_buffer();
    
    for (_,render_surface) in outputs.iter() {        
        let render_pass = &render_surface.test_renderpass;
        let render_target = &render_surface.texture;
        let render_target_view = &render_surface.texture_rtv;

        cmd_buffer
            .cmd_resource_barrier(
                &[],
                &[TextureBarrier::<DefaultApi>::state_transition(
                    render_target,
                    ResourceState::SHADER_RESOURCE|ResourceState::COPY_SRC,
                    ResourceState::RENDER_TARGET,
                )],
            )
            .unwrap();

        render_pass.render(&renderer, cmd_buffer, render_target_view);

        cmd_buffer
            .cmd_resource_barrier(
                &[],
                &[TextureBarrier::<DefaultApi>::state_transition(
                    render_target,
                    ResourceState::RENDER_TARGET,
                    ResourceState::SHADER_RESOURCE|ResourceState::COPY_SRC,
                )],
            )
            .unwrap(); 
    }

    renderer.end_frame();
}
