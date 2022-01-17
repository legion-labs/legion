use lgn_data_runtime::Component;
#[resource()]
struct EntityDc {
    #[legion(default = "new_entity")]
    pub name: String,

    #[legion(hidden, resource_type = EntityDc)]
    pub children: Vec<ResourcePathId>,

    #[legion(ignore_deps, hidden, resource_type = EntityDc)]
    pub parent: Option<ResourcePathId>,

    pub components: Vec<Box<dyn Component>>,
}
