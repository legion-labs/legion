use lgn_core::prelude::Time;
use lgn_ecs::prelude::{Query, Res, ResMut};
use lgn_tracing::prelude::{error, span_fn};
use lgn_transform::prelude::{GlobalTransform, Transform};
use physx::prelude::{Owner, RigidActor, RigidDynamic, Scene, ScratchBuffer};

use crate::PxScene;

#[span_fn]
pub(crate) fn step_simulation(mut scene: ResMut<'_, Owner<PxScene>>, time: Res<'_, Time>) {
    let delta_time = time.delta_seconds();
    if delta_time <= 0_f32 {
        return;
    }

    let mut scratch = create_scratch_buffer();

    if let Err(error) = scene.step(
        delta_time,
        None::<&mut physx_sys::PxBaseTask>,
        Some(&mut scratch),
        true,
    ) {
        error!("error occurred during physics step: {}", error);
    }

    drop(scene);
    drop(time);
}

#[span_fn]
pub(crate) fn sync_transforms(
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

fn create_scratch_buffer() -> ScratchBuffer {
    #[allow(unsafe_code)]
    unsafe {
        ScratchBuffer::new(4)
    }
}
