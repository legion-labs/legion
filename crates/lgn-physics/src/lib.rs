//! Physics plugin

mod labels;
pub use labels::*;

use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use physx::prelude::*;

type PxAllocator = physx::foundation::DefaultAllocator;
type PxMaterial = physx::material::PxMaterial<()>;
type PxShape = physx::shape::PxShape<(), PxMaterial>;

#[derive(Default)]
pub struct PhysicsPlugin {}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_before(
            CoreStage::Update,
            PhysicsStage::Update,
            SystemStage::parallel(),
        );

        app.add_startup_system(Self::setup);
        app.add_system_to_stage(PhysicsStage::Update, Self::update);
    }
}

impl PhysicsPlugin {
    fn setup(mut commands: Commands<'_, '_>) {
        // Holds a PxFoundation and a PxPhysics.
        // Also has an optional Pvd and transport, not enabled by default.
        // The default allocator is the one provided by PhysX.
        let physics = PhysicsFoundation::<PxAllocator, PxShape>::default();

        commands.insert_resource(physics);
    }

    fn update(physics: Res<'_, PhysicsFoundation<PxAllocator, PxShape>>) {
        drop(physics);
    }
}
