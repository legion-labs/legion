use lgn_data_runtime::Component;
#[resource()]
struct EntityDc {
    #[legion(hidden, resource_type = EntityDc)]
    pub children: Vec<ResourcePathId>,

    #[legion(ignore_deps, hidden, resource_type = EntityDc)]
    pub parent: Option<ResourcePathId>,

    pub components: Vec<Box<dyn Component>>,
}
