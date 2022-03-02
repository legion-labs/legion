pub enum RigidActorType {
    Static,
    Dynamic,
}

pub enum CollisionGeometry {
    Sphere { radius: f32 },
    Capsule { radius: f32, half_height: f32 },
    Box { half_extents: Vec3 },
}

#[component]
struct PhysicsRigidActor {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,

    pub collision_geometry: CollisionGeometry,
}
