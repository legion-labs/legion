#[component()]
struct Physics {
    dynamic: bool,

    #[legion(resource_type=crate::runtime::Mesh)]
    collision_geometry: Option<ResourcePathId>,
}
