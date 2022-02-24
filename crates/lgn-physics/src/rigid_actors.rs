use lgn_ecs::prelude::*;
use lgn_graphics_data::DefaultMeshType;
use lgn_transform::prelude::*;
use physx::{foundation::DefaultAllocator, prelude::*};

use crate::{runtime::PhysicsRigidActor, PxMaterial, PxScene, PxShape, RigidActorType};

#[derive(Component)]
pub(crate) struct RigidDynamicActor {
    geometry: PxBoxGeometry,
}

impl RigidDynamicActor {
    pub(crate) fn new(rigid_actor: &PhysicsRigidActor, transform: &GlobalTransform) -> Self {
        debug_assert!(rigid_actor.actor_type == RigidActorType::Dynamic);
        assert!(rigid_actor.collision_mesh_type == DefaultMeshType::Cube);
        assert!(rigid_actor.collision_mesh.is_none());
        let extents = transform.scale / 2_f32;
        Self {
            geometry: PxBoxGeometry::new(extents.x, extents.y, extents.z),
        }
    }

    pub(crate) fn add_actor_to_scene(
        &self,
        physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        scene: &mut ResMut<'_, Owner<PxScene>>,
        transform: &GlobalTransform,
        entity: Entity,
        material: &mut ResMut<'_, Owner<PxMaterial>>,
    ) {
        let transform: PxTransform = transform.compute_matrix().into();
        let mut actor = physics
            .create_rigid_dynamic(
                transform,
                &self.geometry,
                material,
                10_f32,
                PxTransform::default(),
                entity,
            )
            .unwrap();
        actor.set_angular_damping(0.5);
        scene.add_dynamic_actor(actor);
    }
}

#[derive(Component)]
pub(crate) struct RigidStaticActor {
    //geometry: Box<dyn Geometry + Send + Sync>,
    geometry: PxBoxGeometry,
}

impl RigidStaticActor {
    pub(crate) fn new(rigid_actor: &PhysicsRigidActor, transform: &GlobalTransform) -> Self {
        debug_assert!(rigid_actor.actor_type == RigidActorType::Static);
        assert!(rigid_actor.collision_mesh_type == DefaultMeshType::Cube);
        assert!(rigid_actor.collision_mesh.is_none());
        let extents = transform.scale / 2_f32;
        Self {
            geometry: PxBoxGeometry::new(extents.x, extents.y, extents.z),
        }
    }

    pub(crate) fn add_actor_to_scene(
        &self,
        physics: &mut ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        scene: &mut ResMut<'_, Owner<PxScene>>,
        transform: &GlobalTransform,
        entity: Entity,
        material: &mut ResMut<'_, Owner<PxMaterial>>,
    ) {
        let transform: PxTransform = transform.compute_matrix().into();
        let actor = physics
            .create_rigid_static(
                transform,
                &self.geometry,
                material,
                PxTransform::default(),
                entity,
            )
            .unwrap();
        scene.add_static_actor(actor);
    }
}
