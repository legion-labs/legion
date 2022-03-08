use lgn_ecs::prelude::*;
use lgn_transform::prelude::GlobalTransform;
use physx::{
    convex_mesh::ConvexMesh,
    cooking::{ConvexMeshCookingResult, PxConvexMeshDesc, PxCooking},
    foundation::DefaultAllocator,
    prelude::*,
    traits::Class,
};
use physx_sys::{PxConvexFlag, PxConvexMeshGeometryFlags, PxMeshScale};

use crate::{mesh_scale::MeshScale, runtime, PxMaterial, PxScene, PxShape};

pub(crate) enum CollisionGeometry {
    Box(PxBoxGeometry),
    Capsule(PxCapsuleGeometry),
    ConvexMesh(PxConvexMeshGeometry),
    //HeightField(PxHeightFieldGeometry),
    Plane(PxPlaneGeometry),
    Sphere(PxSphereGeometry),
    //TriangleMesh(PxTriangleMeshGeometry),
    None,
}

// SAFETY: the geometry is created when the physics component are parsed, and then immutable
#[allow(unsafe_code)]
unsafe impl Send for CollisionGeometry {}
#[allow(unsafe_code)]
unsafe impl Sync for CollisionGeometry {}

enum CollisionMesh {
    ConvexMesh(Owner<ConvexMesh>),
    None,
}

#[derive(Component)]
pub(crate) struct CollisionComponent {
    pub(crate) geometry: CollisionGeometry,
    mesh: CollisionMesh,
}

pub(crate) trait Convert {
    fn convert(
        &self,
        physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionComponent;
}

impl Convert for runtime::PhysicsRigidBox {
    fn convert(
        &self,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionComponent {
        CollisionComponent {
            geometry: CollisionGeometry::Box(PxBoxGeometry::new(
                self.half_extents.x,
                self.half_extents.y,
                self.half_extents.z,
            )),
            mesh: CollisionMesh::None,
        }
    }
}

impl Convert for runtime::PhysicsRigidCapsule {
    fn convert(
        &self,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionComponent {
        CollisionComponent {
            geometry: CollisionGeometry::Capsule(PxCapsuleGeometry::new(
                self.radius,
                self.half_height,
            )),
            mesh: CollisionMesh::None,
        }
    }
}

impl Convert for runtime::PhysicsRigidConvexMesh {
    #[allow(clippy::fn_to_numeric_cast_with_truncation)]
    fn convert(
        &self,
        physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionComponent {
        let vertices: Vec<PxVec3> = self.vertices.iter().map(|v| (*v).into()).collect();
        let mut mesh_desc = PxConvexMeshDesc::new();
        mesh_desc.obj.points.data = vertices.as_ptr().cast::<std::ffi::c_void>();
        mesh_desc.obj.points.count = vertices.len() as u32;
        mesh_desc.obj.points.stride = std::mem::size_of::<PxVec3>() as u32;
        mesh_desc.obj.flags.mBits = PxConvexFlag::eCOMPUTE_CONVEX as u16;

        // can't validate yet, since convex hull is not computed
        //assert!(cooking.validate_convex_mesh(&mesh_desc));

        let cooking_result = cooking.create_convex_mesh(physics.physics_mut(), &mesh_desc);

        match cooking_result {
            ConvexMeshCookingResult::Success(convex_mesh) => {
                let mut result = CollisionComponent {
                    geometry: CollisionGeometry::None,
                    mesh: CollisionMesh::ConvexMesh(convex_mesh),
                };

                if let CollisionMesh::ConvexMesh(mesh) = &mut result.mesh {
                    let mesh_scale: PxMeshScale = (&self.scale).into();
                    let flags = PxConvexMeshGeometryFlags { mBits: 0 };
                    result.geometry = CollisionGeometry::ConvexMesh(PxConvexMeshGeometry::new(
                        mesh,
                        &mesh_scale,
                        flags,
                    ));
                }

                result
            }
            _ => {
                panic!("mesh cooking failed");
            }
        }
    }
}

impl Convert for runtime::PhysicsRigidPlane {
    fn convert(
        &self,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionComponent {
        CollisionComponent {
            geometry: CollisionGeometry::Plane(PxPlaneGeometry::new()),
            mesh: CollisionMesh::None,
        }
    }
}

impl Convert for runtime::PhysicsRigidSphere {
    fn convert(
        &self,
        _physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        _cooking: &Res<'_, Owner<PxCooking>>,
    ) -> CollisionComponent {
        CollisionComponent {
            geometry: CollisionGeometry::Sphere(PxSphereGeometry::new(self.radius)),
            mesh: CollisionMesh::None,
        }
    }
}

#[allow(unsafe_code)]
unsafe impl Class<PxGeometry> for CollisionGeometry {
    fn as_ptr(&self) -> *const PxGeometry {
        match self {
            Self::Box(geometry) => geometry.as_ptr(),
            Self::Capsule(geometry) => geometry.as_ptr(),
            Self::ConvexMesh(geometry) => geometry.as_ptr(),
            // Self::HeightField(geometry) => geometry.as_ptr(),
            Self::Plane(geometry) => geometry.as_ptr(),
            Self::Sphere(geometry) => geometry.as_ptr(),
            // Self::TriangleMesh(geometry) => geometry.as_ptr(),
            CollisionGeometry::None => panic!("empty geometry"),
        }
    }

    fn as_mut_ptr(&mut self) -> *mut PxGeometry {
        match self {
            Self::Box(geometry) => geometry.as_mut_ptr(),
            Self::Capsule(geometry) => geometry.as_mut_ptr(),
            Self::ConvexMesh(geometry) => geometry.as_mut_ptr(),
            // Self::HeightField(geometry) => geometry.as_mut_ptr(),
            Self::Plane(geometry) => geometry.as_mut_ptr(),
            Self::Sphere(geometry) => geometry.as_mut_ptr(),
            // Self::TriangleMesh(geometry) => geometry.as_mut_ptr(),
            CollisionGeometry::None => panic!("empty geometry"),
        }
    }
}

impl From<&runtime::MeshScale> for PxMeshScale {
    fn from(value: &runtime::MeshScale) -> Self {
        let scale: PxVec3 = value.scale.into();
        let rotation: PxQuat = value.rotation.into();
        Self::new(&scale, &rotation)
    }
}

pub(crate) fn add_dynamic_actor_to_scene(
    physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
    scene: &mut ResMut<'_, Owner<PxScene>>,
    transform: &GlobalTransform,
    geometry: &impl Geometry,
    entity: Entity,
    material: &mut ResMut<'_, Owner<PxMaterial>>,
) {
    debug_assert!(geometry.get_type() != GeometryType::Plane);
    let transform: PxTransform = transform.compute_matrix().into();
    let mut actor = physics
        .create_rigid_dynamic(
            transform,
            geometry,
            material,
            10_f32,
            PxTransform::default(),
            entity,
        )
        .unwrap();
    actor.set_angular_damping(0.5);
    scene.add_dynamic_actor(actor);
}

pub(crate) fn add_static_actor_to_scene(
    physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
    scene: &mut ResMut<'_, Owner<PxScene>>,
    transform: &GlobalTransform,
    geometry: &impl Geometry,
    entity: Entity,
    material: &mut ResMut<'_, Owner<PxMaterial>>,
) {
    let transform: PxTransform = transform.compute_matrix().into();
    let actor = physics
        .create_rigid_static(
            transform,
            geometry,
            material,
            PxTransform::default(),
            entity,
        )
        .unwrap();
    scene.add_static_actor(actor);
}
