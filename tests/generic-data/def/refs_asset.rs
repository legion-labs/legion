#[resource]
#[legion(offline_only)]
struct RefsAsset {
    pub content: String,
    #[legion(resource_type = crate::runtime::RefsAsset)]
    pub reference: Option<ResourcePathId>,
}
