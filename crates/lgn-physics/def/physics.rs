use lgn_graphics_data::DefaultMeshType;

pub enum RigidActorType {
    Static,
    Dynamic,
}

#[component]
struct PhysicsRigidActor {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,

    #[legion(default = DefaultMeshType::Cube)]
    pub collision_mesh_type: DefaultMeshType,

    #[legion(resource_type = lgn_graphics_data::runtime::Model, default = None)]
    pub collision_mesh: Option<ResourcePathId>,
}
