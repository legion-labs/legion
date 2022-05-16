#[resource()]
#[derive(Clone)]
struct Instance {
    #[legion(resource_type=lgn_graphics_data::runtime::Entity)]
    original: Option<ResourcePathId>,
}
