use crate::{
    app::{App, AppExit},
    plugin::Plugin,
    ManualEventReader,
};
use legion_ecs::event::Events;
use legion_utils::{Duration, Instant};

/// Determines the method used to run an [App]'s `Schedule`
#[derive(Copy, Clone, Debug)]
pub enum RunMode {
    Loop { wait: Option<Duration> },
    Once,
}

impl Default for RunMode {
    fn default() -> Self {
        RunMode::Loop { wait: None }
    }
}

#[derive(Copy, Clone, Default)]
pub struct ScheduleRunnerSettings {
    pub run_mode: RunMode,
}

impl ScheduleRunnerSettings {
    pub fn run_once() -> Self {
        ScheduleRunnerSettings {
            run_mode: RunMode::Once,
        }
    }

    pub fn run_loop(wait_duration: Duration) -> Self {
        ScheduleRunnerSettings {
            run_mode: RunMode::Loop {
                wait: Some(wait_duration),
            },
        }
    }
}

/// Configures an App to run its [Schedule](legion_ecs::schedule::Schedule) according to a given
/// [RunMode]
#[derive(Default)]
pub struct ScheduleRunnerPlugin;

impl Plugin for ScheduleRunnerPlugin {
    fn build(&self, app: &mut App) {
        let settings = app
            .world
            .get_resource_or_insert_with(ScheduleRunnerSettings::default)
            .to_owned();
        app.set_runner(move |mut app: App| {
            let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();
            match settings.run_mode {
                RunMode::Once => {
                    app.update();
                }
                RunMode::Loop { wait } => {
                    let mut tick = move |app: &mut App,
                                         wait: Option<Duration>|
                          -> Result<Option<Duration>, AppExit> {
                        let start_time = Instant::now();

                        if let Some(app_exit_events) =
                            app.world.get_resource_mut::<Events<AppExit>>()
                        {
                            if let Some(exit) = app_exit_event_reader.iter(&app_exit_events).last()
                            {
                                return Err(exit.clone());
                            }
                        }

                        app.update();

                        if let Some(app_exit_events) =
                            app.world.get_resource_mut::<Events<AppExit>>()
                        {
                            if let Some(exit) = app_exit_event_reader.iter(&app_exit_events).last()
                            {
                                return Err(exit.clone());
                            }
                        }

                        let end_time = Instant::now();

                        if let Some(wait) = wait {
                            let exe_time = end_time - start_time;
                            if exe_time < wait {
                                return Ok(Some(wait - exe_time));
                            }
                        }

                        Ok(None)
                    };

                    {
                        while let Ok(delay) = tick(&mut app, wait) {
                            if let Some(delay) = delay {
                                std::thread::sleep(delay);
                            }
                        }
                    }
                }
            }
        });
    }
}
