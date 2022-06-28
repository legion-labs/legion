#[resource]
#[legion(offline_only)]
struct NullResource {
    content: isize,

    #[legion(resource_type = crate::runtime::NullResource)]
    dependencies: Vec<ResourcePathId>,
}
