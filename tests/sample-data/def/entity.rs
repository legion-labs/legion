use lgn_data_runtime::Component;

#[resource()]
struct Entity {
    #[legion(hidden, resource_type = Entity)]
    pub children: Vec<ResourcePathId>,

    #[legion(ignore_deps, hidden, resource_type = Entity)]
    pub parent: Option<ResourcePathId>,

    pub components: Vec<Box<dyn Component>>,
}
