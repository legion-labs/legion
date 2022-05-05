use lgn_ecs::prelude::{Commands, Component, Entity, Query, Res};

use crate::time::{Time, Timer};
use instant::Duration;

/// You can attach an `AutoDestruct` component to an entity to have it despawn
/// automatically after an elapsed delay.
#[derive(Component)]
pub struct AutoDestruct {
    timer: Timer,
}

impl AutoDestruct {
    /// Construct an `AutoDestruct` instance set to despawn its entity once duration has elapsed.
    pub fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, false),
        }
    }

    /// Construct an `AutoDestruct` instance set to despawn its entity once duration has elapsed.
    /// Duration is specified in seconds.
    pub fn from_seconds(duration: f32) -> Self {
        Self::new(Duration::from_secs_f32(duration))
    }
}

pub(crate) fn tick_auto_destruct(
    mut commands: Commands<'_, '_>,
    mut query: Query<'_, '_, (Entity, &mut AutoDestruct)>,
    time: Res<'_, Time>,
) {
    for (entity, mut auto_destruct) in query.iter_mut() {
        auto_destruct.timer.tick(time.delta());
        if auto_destruct.timer.finished() {
            commands.entity(entity).despawn();
        }
    }

    drop(time);
}
