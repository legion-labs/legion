use lgn_data_runtime::Component;

#[resource()]
#[derive(Clone)]
struct Entity {
    #[legion(resource_type = Entity)]
    pub children: Vec<ResourcePathId>,

    #[legion(ignore_deps, hidden, resource_type = Entity)]
    pub parent: Option<ResourcePathId>,

    pub components: Vec<Box<dyn Component>>,
}
