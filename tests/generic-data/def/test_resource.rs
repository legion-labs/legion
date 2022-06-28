#[resource]
#[legion(offline_only)]
pub struct TestResource {
    /// Resource's content.
    pub content: String,
    /// Resource's build dependencies.
    #[legion(resource_type = crate::runtime::TestResource)]
    pub build_deps: Vec<ResourcePathId>,
}
