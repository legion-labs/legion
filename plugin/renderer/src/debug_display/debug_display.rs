use crate::RenderContext;

#[derive(Default)]
pub struct DebugDisplay<'a>{
    display_list: Vec<&'a DisplayList>,
}

impl DebugDisplay<'_> {
    pub fn create_display_list(&mut self, render_context: &mut RenderContext) -> &mut DisplayList {
        let bump_allocator = render_context.acquire_bump_allocator();
        bump_allocator.alloc::<DisplayList>()
        unimplemented!()
    }
}


pub struct Cube {

}
pub struct DisplayList {
    
}

impl DisplayList {
    pub fn add_cube(&mut self) {}
    pub fn add_sphere(&mut self) {}
}
