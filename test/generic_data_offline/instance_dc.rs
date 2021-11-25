#[data_container()]
struct InstanceDc {
    #[legion(resource_type = super::entity_dc::EntityDc)]
    pub original: Option<ResourcePathId>,
}
