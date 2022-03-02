use lgn_math::prelude::{Quat, Vec3};

enum RigidActorType {
    Static,
    Dynamic,
}

#[derive(Clone)]
struct MeshScale {
    scale: Vec3,
    rotation: Quat,
}

// #[component]
// struct PhysicsRigidActor {
//     #[legion(default = RigidActorType::Dynamic)]
//     pub actor_type: RigidActorType,

//     pub collision_geometry: CollisionGeometry,
// }

#[component]
struct PhysicsRigidSphere {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,
    // [PhysX sphere geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#spheres)
    radius: f32,
}

#[component]
struct PhysicsRigidCapsule {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,
    // [PhysX capsule geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#capsules)
    radius: f32,
    half_height: f32,
}

#[component]
struct PhysicsRigidBox {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,
    // [PhysX box geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#boxes)
    half_extents: Vec3,
}

#[component]
struct PhysicsRigidPlane {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,
    // [PhysX plane geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#planes)
}

// #[component]
// struct PhysicsRigidConvexMesh {
//     #[legion(default = RigidActorType::Dynamic)]
//     pub actor_type: RigidActorType,
//     // [PhysX convex mesh geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#convex-meshes)
//     scale: MeshScale,
// }

// #[component]
// struct PhysicsRigidTriangleMesh {
//     #[legion(default = RigidActorType::Dynamic)]
//     pub actor_type: RigidActorType,
//     // [PhysX triangle mesh geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#triangle-meshes)
//     scale: MeshScale,
// }

#[component]
struct PhysicsRigidHeightField {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,
    // [PhysX height field geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#height-fields)
    height_scale: f32,
    row_scale: f32,
    column_scale: f32,
}
