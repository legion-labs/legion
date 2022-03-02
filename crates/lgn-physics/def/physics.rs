use lgn_math::prelude::{Quat, Vec3};

enum RigidActorType {
    Static,
    Dynamic,
}

struct MeshScale {
    scale: Vec3,
    rotation: Quat,
}

enum CollisionGeometry {
    // [PhysX sphere geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#spheres)
    Sphere {
        radius: f32,
    },
    // [PhysX capsule geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#capsules)
    Capsule {
        radius: f32,
        half_height: f32,
    },
    // [PhysX box geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#boxes)
    Box {
        half_extents: Vec3,
    },
    // [PhysX plane geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#planes)
    Plane,
    // [PhysX convex mesh geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#convex-meshes)
    ConvexMesh {
        scale: MeshScale,
    },
    // [PhysX triangle mesh geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#triangle-meshes)
    TriangleMesh {
        scale: MeshScale,
    },
    // [PhysX height field geometry](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#height-fields)
    HeightField {
        height_scale: f32,
        row_scale: f32,
        column_scale: f32,
    },
}

#[component]
struct PhysicsRigidActor {
    #[legion(default = RigidActorType::Dynamic)]
    pub actor_type: RigidActorType,

    pub collision_geometry: CollisionGeometry,
}
