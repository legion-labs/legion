#[component()]
struct Visual {
    #[legion(resource_type = lgn_graphics_data::runtime::Model)]
    pub renderable_geometry: Option<ResourcePathId>,

    pub shadow_receiver: bool,
    pub shadow_caster_sun: bool,
    pub shadow_caster_local: bool,
    pub gi_contribution: usize,
}

/*
pub enum GIContribution {
    Default,
    Blocker,
    Exclude,
}
*/
