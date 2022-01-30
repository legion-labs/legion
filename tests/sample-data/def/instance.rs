#[resource()]
struct Instance {
    #[legion(resource_type=crate::runtime::Entity)]
    original: Option<ResourcePathId>,
}
