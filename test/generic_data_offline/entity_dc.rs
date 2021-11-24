use lgn_data_runtime::Component;

#[resource()]
struct EntityDc {
    #[legion(default = "unnamed")]
    pub name: String,

    //#[legion(resource_type = EntityDc)]
    //pub children: Vec<ResourcePathId>,

    //#[legion(resource_type = EntityDc)]
    //pub parent: Option<ResourcePathId>,
    pub components: Vec<Box<dyn Component>>,
}
