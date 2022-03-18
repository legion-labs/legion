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

mod actor_type;
mod callbacks;
mod collision_geometry;
mod debug_display;
mod labels;
mod mesh_scale;
mod physics_options;
mod rigid_actors;
mod settings;
mod simulation;

use lgn_app::prelude::{App, CoreStage, Plugin};
use lgn_ecs::prelude::{Commands, Entity, Query, Res, ResMut, SystemStage};
use lgn_graphics_renderer::labels::RenderStage;
use lgn_tracing::prelude::{error, warn};
use physx::{
    cooking::{PxCooking, PxCookingParams},
    foundation::DefaultAllocator,
    physics::PhysicsFoundationBuilder,
    prelude::{Owner, Physics, PhysicsFoundation, Scene, SceneDescriptor},
};
use physx_sys::{PxPvdInstrumentationFlag, PxPvdInstrumentationFlags};

use crate::{
    actor_type::WithActorType,
    callbacks::{OnAdvance, OnCollision, OnConstraintBreak, OnTrigger, OnWakeSleep},
    collision_geometry::ConvertToCollisionGeometry,
    debug_display::display_collision_geometry,
    physics_options::PhysicsOptions,
    rigid_actors::create_rigid_actors,
    simulation::{step_simulation, sync_transforms},
};
pub use crate::{
    labels::PhysicsStage,
    settings::{PhysicsSettings, PhysicsSettingsBuilder},
};

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
    bool,
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

        app.add_system_to_stage(PhysicsStage::Update, Self::process_scene_settings);

        app.add_system_to_stage(
            PhysicsStage::Update,
            create_rigid_actors::<runtime::PhysicsRigidBox>,
        )
        .add_system_to_stage(
            PhysicsStage::Update,
            create_rigid_actors::<runtime::PhysicsRigidCapsule>,
        )
        .add_system_to_stage(
            PhysicsStage::Update,
            create_rigid_actors::<runtime::PhysicsRigidConvexMesh>,
        )
        // app.add_system_to_stage(
        //     PhysicsStage::Update,
        //     create_rigid_actors::<runtime::PhysicsRigidHeightField>,
        // );
        .add_system_to_stage(
            PhysicsStage::Update,
            create_rigid_actors::<runtime::PhysicsRigidPlane>,
        )
        .add_system_to_stage(
            PhysicsStage::Update,
            create_rigid_actors::<runtime::PhysicsRigidSphere>,
        )
        .add_system_to_stage(
            PhysicsStage::Update,
            create_rigid_actors::<runtime::PhysicsRigidTriangleMesh>,
        );

        app.add_system_to_stage(PhysicsStage::Update, step_simulation)
            .add_system_to_stage(PhysicsStage::Update, sync_transforms);

        app.init_resource::<PhysicsOptions>()
            .add_system_to_stage(RenderStage::Prepare, physics_options::ui_physics_options)
            .add_system_to_stage(RenderStage::Prepare, display_collision_geometry);
    }
}

impl PhysicsPlugin {
    fn setup(settings: Res<'_, PhysicsSettings>, mut commands: Commands<'_, '_>) {
        let mut physics = Self::create_physics_foundation(
            settings.enable_visual_debugger,
            settings.length_tolerance,
            settings.speed_tolerance,
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
                    physics = Self::create_physics_foundation(
                        false,
                        settings.length_tolerance,
                        settings.speed_tolerance,
                    );
                    if physics.is_some() {
                        warn!("failed to connect to physics visual debugger");
                    }
                }
            }
        }
        let mut physics = physics.expect("failed to create physics foundation");

        // physics cooking, for runtime mesh creation
        let cooking_params =
            PxCookingParams::new(&physics).expect("failed to create physics cooking params");
        let cooking = PxCooking::new(physics.foundation_mut(), &cooking_params)
            .expect("failed to create physics cooking module");

        let scene: Owner<PxScene> = physics
            .create(SceneDescriptor {
                gravity: settings.gravity.into(),
                on_advance: Some(OnAdvance),
                ..SceneDescriptor::new(false)
            })
            .expect("failed to create physics scene");

        if let Some(default_material) = physics.create_material(0.5, 0.5, 0.6, ()) {
            commands.insert_resource(default_material);
        }

        commands.insert_resource(scene);
        commands.insert_resource(cooking);

        // Note: important to insert physics after scene, for drop order
        commands.insert_resource(physics);

        commands.remove_resource::<PhysicsSettings>(); // no longer needed
        drop(settings);
    }

    fn process_scene_settings(
        query: Query<'_, '_, (Entity, &runtime::PhysicsSceneSettings)>,
        mut scene: ResMut<'_, Owner<PxScene>>,
        mut commands: Commands<'_, '_>,
    ) {
        let mut are_settings_already_set = *scene.get_user_data();

        for (entity, scene_settings) in query.iter() {
            if !are_settings_already_set {
                scene.set_gravity(
                    scene_settings.gravity.x,
                    scene_settings.gravity.y,
                    scene_settings.gravity.z,
                );
                are_settings_already_set = true;
                #[allow(unsafe_code)]
                unsafe {
                    *scene.get_user_data_mut() = true;
                }
            } else {
                error!(
                    "physics scene settings already set, ignoring additional settings in entity {}",
                    entity.id()
                );
            }
            commands
                .entity(entity)
                .remove::<runtime::PhysicsSceneSettings>();
        }

        drop(query);
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
}
