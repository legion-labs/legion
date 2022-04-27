use instant::{Duration, Instant};
use lgn_ecs::system::ResMut;

/// Tracks elapsed time since the last update and since the App has started
#[derive(Debug, Clone)]
pub struct Time {
    delta: Duration,
    last_update: Option<Instant>,
    delta_seconds_f64: f64,
    delta_seconds: f32,
    seconds_since_startup: f64,
    time_since_startup: Duration,
    startup: Instant,
    frame_counter: usize,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta: Duration::from_secs(0),
            last_update: None,
            startup: Instant::now(),
            delta_seconds_f64: 0.0,
            seconds_since_startup: 0.0,
            time_since_startup: Duration::from_secs(0),
            delta_seconds: 0.0,
            frame_counter: 0,
        }
    }
}

impl Time {
    /// Updates the internal time measurements.
    ///
    /// Calling this method on the [`Time`] resource as part of your app will most likely result in
    /// inaccurate timekeeping, as the resource is ordinarily managed by the
    /// [`CorePlugin`](crate::CorePlugin).
    pub fn update(&mut self) {
        self.update_with_instant(Instant::now());
    }

    /// Update time with a specified [`Instant`]
    ///
    /// This method is provided for use in tests. Calling this method on the [`Time`] resource as
    /// part of your app will most likely result in inaccurate timekeeping, as the resource is
    /// ordinarily managed by the [`CorePlugin`](crate::CorePlugin).
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_core::prelude::*;
    /// # use bevy_ecs::prelude::*;
    /// # use bevy_utils::Duration;
    /// # fn main () {
    /// #     test_health_system();
    /// # }
    /// struct Health {
    ///     // Health value between 0.0 and 1.0
    ///     health_value: f32,
    /// }
    ///
    /// fn health_system(time: Res<Time>, mut health: ResMut<Health>) {
    ///     // Increase health value by 0.1 per second, independent of frame rate,
    ///     // but not beyond 1.0
    ///     health.health_value = (health.health_value + 0.1 * time.delta_seconds()).min(1.0);
    /// }
    ///
    /// // Mock time in tests
    /// fn test_health_system() {
    ///     let mut world = World::default();
    ///     let mut time = Time::default();
    ///     time.update();
    ///     world.insert_resource(time);
    ///     world.insert_resource(Health { health_value: 0.2 });
    ///
    ///     let mut update_stage = SystemStage::single_threaded();
    ///     update_stage.add_system(health_system);
    ///
    ///     // Simulate that 30 ms have passed
    ///     let mut time = world.resource_mut::<Time>();
    ///     let last_update = time.last_update().unwrap();
    ///     time.update_with_instant(last_update + Duration::from_millis(30));
    ///
    ///     // Run system
    ///     update_stage.run(&mut world);
    ///
    ///     // Check that 0.003 has been added to the health value
    ///     let expected_health_value = 0.2 + 0.1 * 0.03;
    ///     let actual_health_value = world.resource::<Health>().health_value;
    ///     assert_eq!(expected_health_value, actual_health_value);
    /// }
    /// ```
    pub fn update_with_instant(&mut self, instant: Instant) {
        if let Some(last_update) = self.last_update {
            self.delta = instant - last_update;
            self.delta_seconds_f64 = self.delta.as_secs_f64();
            self.delta_seconds = self.delta.as_secs_f32();
        }

        self.time_since_startup = instant - self.startup;
        self.seconds_since_startup = self.time_since_startup.as_secs_f64();
        self.last_update = Some(instant);
        self.frame_counter += 1;
    }

    /// The delta between the current tick and last tick as a [`Duration`]
    #[inline]
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// The delta between the current and last tick as [`f32`] seconds
    #[inline]
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    /// The delta between the current and last tick as [`f64`] seconds
    #[inline]
    pub fn delta_seconds_f64(&self) -> f64 {
        self.delta_seconds_f64
    }

    /// The time from startup to the last update in seconds
    #[inline]
    pub fn seconds_since_startup(&self) -> f64 {
        self.seconds_since_startup
    }

    /// The [`Instant`] the app was started
    #[inline]
    pub fn startup(&self) -> Instant {
        self.startup
    }

    /// The [`Instant`] when [`Time::update`] was last called, if it exists
    #[inline]
    pub fn last_update(&self) -> Option<Instant> {
        self.last_update
    }

    /// The [`Duration`] from startup to the last update
    #[inline]
    pub fn time_since_startup(&self) -> Duration {
        self.time_since_startup
    }

    /// The number of times that time measurements were updated, i.e. the frame-counter
    pub fn frame_counter(&self) -> usize {
        self.frame_counter
    }
}

pub(crate) fn time_system(mut time: ResMut<'_, Time>) {
    time.update();
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use instant::{Duration, Instant};

    use super::Time;

    #[test]
    fn update_test() {
        let start_instant = Instant::now();

        // Create a `Time` for testing
        let mut time = Time {
            startup: start_instant,
            ..Time::default()
        };

        // Ensure `time` was constructed correctly
        assert_eq!(time.delta(), Duration::from_secs(0));
        assert_eq!(time.last_update(), None);
        assert_eq!(time.startup(), start_instant);
        assert_eq!(time.delta_seconds_f64(), 0.0);
        assert_eq!(time.seconds_since_startup(), 0.0);
        assert_eq!(time.time_since_startup(), Duration::from_secs(0));
        assert_eq!(time.delta_seconds(), 0.0);

        // Update `time` and check results
        let first_update_instant = Instant::now();

        time.update_with_instant(first_update_instant);

        assert_eq!(time.delta(), Duration::from_secs(0));
        assert_eq!(time.last_update(), Some(first_update_instant));
        assert_eq!(time.startup(), start_instant);
        assert_eq!(time.delta_seconds_f64(), 0.0);
        assert_eq!(
            time.seconds_since_startup(),
            (first_update_instant - start_instant).as_secs_f64()
        );
        assert_eq!(
            time.time_since_startup(),
            (first_update_instant - start_instant)
        );
        assert_eq!(time.delta_seconds, 0.0);

        // Update `time` again and check results
        let second_update_instant = Instant::now();

        time.update_with_instant(second_update_instant);

        assert_eq!(time.delta(), second_update_instant - first_update_instant);
        assert_eq!(time.last_update(), Some(second_update_instant));
        assert_eq!(time.startup(), start_instant);
        // At this point its safe to use time.delta as a valid value
        // because it's been previously verified to be correct
        assert_eq!(time.delta_seconds_f64(), time.delta().as_secs_f64());
        assert_eq!(
            time.seconds_since_startup(),
            (second_update_instant - start_instant).as_secs_f64()
        );
        assert_eq!(
            time.time_since_startup(),
            (second_update_instant - start_instant)
        );
        assert_eq!(time.delta_seconds(), time.delta().as_secs_f32());
    }
}
