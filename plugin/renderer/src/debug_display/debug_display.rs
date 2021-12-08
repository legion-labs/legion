#[derive(Default)]
pub struct DebugDisplay {}

impl DebugDisplay {
    pub fn create_display_list(&mut self) -> &mut DisplayList {
        unimplemented!()
    }
}

pub struct DisplayList {}

impl DisplayList {
    pub fn add_cube(&mut self) {}
    pub fn add_sphere(&mut self) {}
}
