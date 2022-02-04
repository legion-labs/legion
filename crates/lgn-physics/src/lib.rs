//! Physics plugin

mod labels;
pub use labels::*;

use lgn_app::prelude::*;
use lgn_core::prelude::*;
use lgn_ecs::prelude::*;
use lgn_tracing::prelude::*;
use lgn_transform::prelude::*;
use physx::{foundation::DefaultAllocator, physics::PhysicsFoundationBuilder, prelude::*};

// type aliases

type PxMaterial = physx::material::PxMaterial<()>;
type PxShape = physx::shape::PxShape<(), PxMaterial>;
type PxArticulationLink = physx::articulation_link::PxArticulationLink<(), PxShape>;
type PxRigidStatic = physx::rigid_static::PxRigidStatic<(), PxShape>;
type PxRigidDynamic = physx::rigid_dynamic::PxRigidDynamic<(), PxShape>;
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
//struct DynamicRigidBodyHandle(PxRigidStatic);

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

        app.add_system_to_stage(PhysicsStage::Update, Self::step_simulation);
        app.add_system_to_stage(PhysicsStage::Update, Self::sync_transforms);
    }
}

impl PhysicsPlugin {
    fn setup(mut commands: Commands<'_, '_>) {
        let mut physics_builder = PhysicsFoundationBuilder::<DefaultAllocator>::default();
        physics_builder
            .enable_visual_debugger(false)
            .set_length_tolerance(1.0)
            .set_speed_tolerance(1.0)
            .with_extensions(false);
        let mut physics = physics_builder.build::<PxShape>().unwrap();

        {
            let scene: Owner<PxScene> = physics
                .create(SceneDescriptor {
                    gravity: PxVec3::new(0.0, -9.81, 0.0),
                    on_advance: Some(OnAdvance),
                    ..SceneDescriptor::new(())
                })
                .unwrap();

            commands.insert_resource(scene);
        }

        // Note: important to insert physics after scene, for drop order
        commands.insert_resource(physics);
    }

    #[span_fn]
    fn step_simulation(mut scene: ResMut<'_, Owner<PxScene>>, time: Res<'_, Time>) {
        let delta_time = time.delta_seconds();
        if delta_time <= 0_f32 {
            return;
        }

        #[allow(unsafe_code)]
        let mut scratch = unsafe { ScratchBuffer::new(4) };

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
    fn sync_transforms(mut query: Query<'_, '_, (&RigidDynamicActor, &mut Transform)>) {
        for (dynamic, mut transform) in query.iter_mut() {
            *transform = Transform::from_matrix(dynamic.actor.get_global_pose().into());
        }
    }

    // fn create_dynamic() {
    //     let mut sphere_actor = physics
    //         .create_rigid_dynamic(
    //             PxTransform::from_translation(&PxVec3::new(0.0, 40.0, 100.0)),
    //             &sphere_geo,
    //             material.as_mut(),
    //             10.0,
    //             PxTransform::default(),
    //             (),
    //         )
    //         .unwrap();
    //     sphere_actor.set_angular_damping(0.5);
    //     sphere_actor.set_rigid_body_flag(RigidBodyFlag::EnablePoseIntegrationPreview, true);
    //     scene.add_dynamic_actor(sphere_actor);
    // }
}

#[derive(Component)]
struct RigidDynamicActor {
    actor: Owner<PxRigidDynamic>,
}

// callbacks

struct OnCollision;
impl CollisionCallback for OnCollision {
    fn on_collision(
        &mut self,
        _header: &physx_sys::PxContactPairHeader,
        _pairs: &[physx_sys::PxContactPair],
    ) {
    }
}
struct OnTrigger;
impl TriggerCallback for OnTrigger {
    fn on_trigger(&mut self, _pairs: &[physx_sys::PxTriggerPair]) {}
}

struct OnConstraintBreak;
impl ConstraintBreakCallback for OnConstraintBreak {
    fn on_constraint_break(&mut self, _constraints: &[physx_sys::PxConstraintInfo]) {}
}
struct OnWakeSleep;
impl WakeSleepCallback<PxArticulationLink, PxRigidStatic, PxRigidDynamic> for OnWakeSleep {
    fn on_wake_sleep(
        &mut self,
        _actors: &[&physx::actor::ActorMap<PxArticulationLink, PxRigidStatic, PxRigidDynamic>],
        _is_waking: bool,
    ) {
    }
}

struct OnAdvance;
impl AdvanceCallback<PxArticulationLink, PxRigidDynamic> for OnAdvance {
    fn on_advance(
        &self,
        _actors: &[&physx::rigid_body::RigidBodyMap<PxArticulationLink, PxRigidDynamic>],
        _transforms: &[PxTransform],
    ) {
    }
}
