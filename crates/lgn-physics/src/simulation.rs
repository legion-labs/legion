use std::ops::{Deref, DerefMut};

use lgn_ecs::prelude::{Query, Res, ResMut};
use lgn_time::prelude::Time;
use lgn_tracing::prelude::error;
use lgn_transform::prelude::{GlobalTransform, Transform};
use physx::prelude::{Owner, RigidActor, RigidDynamic, Scene, ScratchBuffer};

use crate::PxScene;

pub(crate) fn step_simulation(
    mut scene: ResMut<'_, Owner<PxScene>>,
    time: Res<'_, Time>,
    mut memory: ResMut<'_, SimulationMemory>,
) {
    let delta_time = time.delta_seconds();
    if delta_time <= 0_f32 {
        return;
    }

    if let Err(error) = scene.step(
        delta_time,
        None::<&mut physx_sys::PxBaseTask>,
        Some(&mut memory),
        true,
    ) {
        error!("error occurred during physics step: {}", error);
    }

    drop(scene);
    drop(time);
}

pub(crate) fn sync_transforms(
    mut scene: ResMut<'_, Owner<PxScene>>,
    mut query: Query<'_, '_, &mut Transform>,
) {
    for actor in scene.get_dynamic_actors() {
        let entity = *actor.get_user_data();
        if let Ok(mut transform) = query.get_mut(entity) {
            let global_transform = GlobalTransform::from_matrix(actor.get_global_pose().into());
            // TODO: use parent global to determine child local
            *transform = global_transform.into();
        } else {
            error!(
                "dynamic actor without a Transform component, entity {}",
                entity.id()
            );
        }
    }
}

pub(crate) struct SimulationMemory(ScratchBuffer);

impl Default for SimulationMemory {
    fn default() -> Self {
        #[allow(unsafe_code)]
        let scratch_buffer = unsafe { ScratchBuffer::new(4) };
        Self(scratch_buffer)
    }
}

impl Deref for SimulationMemory {
    type Target = ScratchBuffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SimulationMemory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(unsafe_code)]
unsafe impl Send for SimulationMemory {}
#[allow(unsafe_code)]
unsafe impl Sync for SimulationMemory {}
