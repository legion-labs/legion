use lgn_math::prelude::{Quat, Vec3};

enum RigidActorType {
    Static,
    Dynamic,
}

#[component]
struct PhysicsRigidBox {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,
    // [PhysX box geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#boxes)
    half_extents: Vec3,
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
struct PhysicsRigidConvexMesh {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,
    // [PhysX convex mesh geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#convex-meshes)
    vertices: Vec<Vec3>,
    scale: MeshScale,
}

#[component]
struct PhysicsRigidPlane {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,
    // [PhysX plane geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#planes)
}

#[component]
struct PhysicsRigidHeightField {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,
    // [PhysX height field geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#height-fields)
    height_scale: f32,
    row_scale: f32,
    column_scale: f32,
}

#[component]
struct PhysicsRigidSphere {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,
    // [PhysX sphere geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#spheres)
    radius: f32,
}

#[component]
struct PhysicsRigidTriangleMesh {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,
    // [PhysX triangle mesh geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#triangle-meshes)
    scale: MeshScale,
}

struct MeshScale {
    scale: Vec3,
    rotation: Quat,
}
