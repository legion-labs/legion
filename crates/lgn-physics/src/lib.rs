//! Physics plugin for Legion's ECS
//!
//! Provides a high-level interfaces with NVIDIA's `PhysX` library
//!
//! To associate a rigid body collision geometry with an entity,
//! attach one of these components to it (depending on the geometry type):
//!
//! | Component | `PhysX` geometry |
//! | --------- | ---------------- |
//! | [`PhysicsRigidBox`](runtime::PhysicsRigidBox) | [`PxBoxGeometry`](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#boxes) |
//! | [`PhysicsRigidCapsule`](runtime::PhysicsRigidCapsule) | [`PxCapsuleGeometry`](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#capsules) |
//! | [`PhysicsRigidConvexMesh`](runtime::PhysicsRigidConvexMesh) | [`PxConvexMeshGeometry`](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#convex-meshes) |
//! | [`PhysicsRigidPlane`](runtime::PhysicsRigidPlane) | [`PxPlaneGeometry`](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#planes) |
//! | [`PhysicsRigidSphere`](runtime::PhysicsRigidSphere) | [`PxSphereGeometry`](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#spheres) |
//! | [`PhysicsRigidTriangleMesh`](runtime::PhysicsRigidTriangleMesh) | [`PxTriangleMeshGeometry`](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Geometry.html#triangle-meshes) |
//!
//! For each of these components, you can also specify if the actor/entity should be static (immovable) or dynamic (subject to physical forces)
//!
//! References:
//! * [NVIDIA `PhysX` overview](https://developer.nvidia.com/physx-sdk)
//! * [`PhysX` 4.1 SDK Guide](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Index.html)
//! * [`PhysX` Visual Debugger](https://developer.nvidia.com/physx-visual-debugger)
//! * [physx-rs](https://github.com/EmbarkStudios/physx-rs), a Rust wrapper by Embark Studios

// generated from def\physics.rs
include!(concat!(env!("OUT_DIR"), "/data_def.rs"));

mod data_def_ext;

mod actor_type;
use actor_type::WithActorType;

mod callbacks;
use callbacks::{OnAdvance, OnCollision, OnConstraintBreak, OnTrigger, OnWakeSleep};

mod labels;
pub use labels::*;

mod mesh_scale;

mod rigid_actors;
use rigid_actors::{
    add_dynamic_actor_to_scene, add_static_actor_to_scene, CollisionGeometry, Convert,
};

mod settings;
pub use settings::PhysicsSettings;

use lgn_app::prelude::*;
use lgn_core::prelude::*;
use lgn_ecs::prelude::*;
use lgn_tracing::prelude::*;
use lgn_transform::prelude::*;
use physx::{
    cooking::{PxCooking, PxCookingParams},
    foundation::DefaultAllocator,
    physics::PhysicsFoundationBuilder,
    prelude::*,
};
use physx_sys::{PxPvdInstrumentationFlag, PxPvdInstrumentationFlags};

// type aliases

type PxMaterial = physx::material::PxMaterial<()>;
type PxShape = physx::shape::PxShape<(), PxMaterial>;
type PxArticulationLink = physx::articulation_link::PxArticulationLink<(), PxShape>;
type PxRigidStatic = physx::rigid_static::PxRigidStatic<Entity, PxShape>;
type PxRigidDynamic = physx::rigid_dynamic::PxRigidDynamic<Entity, PxShape>;
type PxArticulation = physx::articulation::PxArticulation<(), PxArticulationLink>;
type PxArticulationReducedCoordinate =
    physx::articulation_reduced_coordinate::PxArticulationReducedCoordinate<(), PxArticulationLink>;
type PxScene = physx::scene::PxScene<
    (),
    PxArticulationLink,
    PxRigidStatic,
    PxRigidDynamic,
    PxArticulation,
    PxArticulationReducedCoordinate,
    OnCollision,
    OnTrigger,
    OnConstraintBreak,
    OnWakeSleep,
    OnAdvance,
>;

#[derive(Default)]
pub struct PhysicsPlugin {}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::setup);

        app.add_stage_before(
            CoreStage::Update,
            PhysicsStage::Update,
            SystemStage::parallel(),
        );

        app.add_system_to_stage(
            PhysicsStage::Update,
            Self::create_rigid_actors::<runtime::PhysicsRigidBox>,
        )
        .add_system_to_stage(
            PhysicsStage::Update,
            Self::create_rigid_actors::<runtime::PhysicsRigidCapsule>,
        )
        .add_system_to_stage(
            PhysicsStage::Update,
            Self::create_rigid_actors::<runtime::PhysicsRigidConvexMesh>,
        )
        // app.add_system_to_stage(
        //     PhysicsStage::Update,
        //     Self::create_rigid_actors::<runtime::PhysicsRigidHeightField>,
        // );
        .add_system_to_stage(
            PhysicsStage::Update,
            Self::create_rigid_actors::<runtime::PhysicsRigidPlane>,
        )
        .add_system_to_stage(
            PhysicsStage::Update,
            Self::create_rigid_actors::<runtime::PhysicsRigidSphere>,
        )
        .add_system_to_stage(
            PhysicsStage::Update,
            Self::create_rigid_actors::<runtime::PhysicsRigidTriangleMesh>,
        );

        app.add_system_to_stage(PhysicsStage::Update, Self::step_simulation)
            .add_system_to_stage(PhysicsStage::Update, Self::sync_transforms);
    }
}

impl PhysicsPlugin {
    fn setup(settings: Res<'_, PhysicsSettings>, mut commands: Commands<'_, '_>) {
        let length_tolerance = settings.length_tolerance;
        let speed_tolerance = settings.speed_tolerance;
        let mut physics = Self::create_physics_foundation(
            settings.enable_visual_debugger,
            length_tolerance,
            speed_tolerance,
        );
        if settings.enable_visual_debugger {
            match &mut physics {
                Some(physics) => {
                    if let Some(pvd) = physics.pvd_mut() {
                        if pvd.is_connected(true) {
                            // reconnect with additional flags
                            pvd.disconnect();
                            let flags = PxPvdInstrumentationFlags {
                                mBits: PxPvdInstrumentationFlag::eDEBUG as u8
                                    // | PxPvdInstrumentationFlag::ePROFILE as u8
                                     | PxPvdInstrumentationFlag::eMEMORY as u8,
                            };
                            pvd.connect(flags);
                        }
                    }
                }
                None => {
                    // likely failed to connect to visual debugger, retry without
                    physics =
                        Self::create_physics_foundation(false, length_tolerance, speed_tolerance);
                    if physics.is_some() {
                        error!("failed to connect to physics visual debugger");
                    }
                }
            }
        }
        let mut physics = physics.unwrap();

        // physics cooking, for runtime mesh creation
        let cooking_params = PxCookingParams::new(&physics).unwrap();
        let cooking = PxCooking::new(physics.foundation_mut(), &cooking_params).unwrap();

        let scene: Owner<PxScene> = physics
            .create(SceneDescriptor {
                gravity: PxVec3::new(0.0, -9.81, 0.0),
                on_advance: Some(OnAdvance),
                ..SceneDescriptor::new(())
            })
            .unwrap();

        let default_material = physics.create_material(0.5, 0.5, 0.6, ()).unwrap();
        commands.insert_resource(default_material);

        commands.insert_resource(scene);

        commands.insert_resource(cooking);

        // Note: important to insert physics after scene, for drop order
        commands.insert_resource(physics);

        commands.remove_resource::<PhysicsSettings>(); // no longer needed
        drop(settings);
    }

    fn create_rigid_actors<T>(
        query: Query<'_, '_, (Entity, &T, &GlobalTransform)>,
        mut physics: ResMut<'_, PhysicsFoundation<DefaultAllocator, PxShape>>,
        cooking: Res<'_, Owner<PxCooking>>,
        mut scene: ResMut<'_, Owner<PxScene>>,
        mut default_material: ResMut<'_, Owner<PxMaterial>>,
        mut commands: Commands<'_, '_>,
    ) where
        T: Component + Convert + WithActorType,
    {
        for (entity, physics_component, transform) in query.iter() {
            let geometry: CollisionGeometry = physics_component.convert(&mut physics, &cooking);

            match physics_component.get_actor_type() {
                RigidActorType::Dynamic => {
                    add_dynamic_actor_to_scene(
                        &mut physics,
                        &mut scene,
                        transform,
                        &geometry,
                        entity,
                        &mut default_material,
                    );
                }
                RigidActorType::Static => {
                    add_static_actor_to_scene(
                        &mut physics,
                        &mut scene,
                        transform,
                        &geometry,
                        entity,
                        &mut default_material,
                    );
                }
            }

            commands.entity(entity).insert(geometry).remove::<T>();
        }

        drop(query);
        drop(cooking);
    }

    #[span_fn]
    fn step_simulation(mut scene: ResMut<'_, Owner<PxScene>>, time: Res<'_, Time>) {
        let delta_time = time.delta_seconds();
        if delta_time <= 0_f32 {
            return;
        }

        let mut scratch = Self::create_scratch_buffer();

        scene
            .step(
                delta_time,
                None::<&mut physx_sys::PxBaseTask>,
                Some(&mut scratch),
                true,
            )
            .expect("error occurred during simulation");

        drop(scene);
        drop(time);
    }

    #[span_fn]
    fn sync_transforms(
        mut scene: ResMut<'_, Owner<PxScene>>,
        mut query: Query<'_, '_, &mut Transform>,
    ) {
        for actor in scene.get_dynamic_actors() {
            let entity = actor.get_user_data();
            if let Ok(mut transform) = query.get_mut(*entity) {
                let global_transform = GlobalTransform::from_matrix(actor.get_global_pose().into());
                // TODO: use parent global to determine child local
                *transform = global_transform.into();
            }
        }
    }

    fn create_physics_foundation(
        enable_visual_debugger: bool,
        length_tolerance: f32,
        speed_tolerance: f32,
    ) -> Option<PhysicsFoundation<DefaultAllocator, PxShape>> {
        let mut physics_builder = PhysicsFoundationBuilder::<DefaultAllocator>::default();
        physics_builder
            .enable_visual_debugger(enable_visual_debugger)
            .set_length_tolerance(length_tolerance)
            .set_speed_tolerance(speed_tolerance)
            .with_extensions(false);
        physics_builder.build()
    }

    fn create_scratch_buffer() -> ScratchBuffer {
        #[allow(unsafe_code)]
        unsafe {
            ScratchBuffer::new(4)
        }
    }
}
