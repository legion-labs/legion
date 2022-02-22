//! Physics plugin
//! Interfaces with NVIDIA's `PhysX` library
//! Reference: [`PhysX` 4.1 SDK Guide](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Index.html)

mod labels;
pub use labels::*;

mod callbacks;
use callbacks::{OnAdvance, OnCollision, OnConstraintBreak, OnTrigger, OnWakeSleep};

mod rigid_dynamic;
use rigid_dynamic::RigidDynamicActor;

mod settings;
pub use settings::PhysicsSettings;

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
    fn setup(settings: Res<'_, PhysicsSettings>, mut commands: Commands<'_, '_>) {
        let mut physics_builder = PhysicsFoundationBuilder::<DefaultAllocator>::default();
        physics_builder
            .enable_visual_debugger(settings.enable_visual_debugger)
            .set_length_tolerance(settings.length_tolerance)
            .set_speed_tolerance(settings.speed_tolerance)
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

        commands.remove_resource::<PhysicsSettings>(); // no longer needed
        drop(settings);
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
    fn sync_transforms(mut query: Query<'_, '_, (&RigidDynamicActor, &mut Transform)>) {
        for (dynamic, mut transform) in query.iter_mut() {
            *transform = Transform::from_matrix(dynamic.actor.get_global_pose().into());
        }
    }

    fn create_scratch_buffer() -> ScratchBuffer {
        #[allow(unsafe_code)]
        unsafe {
            ScratchBuffer::new(4)
        }
    }
}
