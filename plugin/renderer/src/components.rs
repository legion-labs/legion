use legion_ecs::prelude::Component;
use legion_window::WindowId;

pub struct RenderOutputId(u64);

#[derive(Component)]
pub struct RenderOutput {
    pub id : WindowId,
    pub width : u32,
    pub height : u32,

}

