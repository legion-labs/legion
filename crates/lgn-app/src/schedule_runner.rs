use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

use instant::{Duration, Instant};
use lgn_ecs::event::Events;
use lgn_tracing::{imetric, info, span_scope};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::*, JsCast};

use crate::{
    app::{App, AppExit},
    plugin::Plugin,
    ManualEventReader,
};

/// Determines the method used to run an [`App`]'s [`Schedule`](lgn_ecs::schedule::Schedule).
///
/// It is used in the [`ScheduleRunnerSettings`].
#[derive(Copy, Clone, Debug)]
pub enum RunMode {
    /// Indicates that the [`App`]'s schedule should run repeatedly.
    Loop {
        /// The minimum [`Duration`] to wait after a [`Schedule`](bevy_ecs::schedule::Schedule)
        /// has completed before repeating. A value of [`None`] will not wait.
        wait: Option<Duration>,
    },
    /// Indicates that the [`App`]'s schedule should run only once.
    Once,
}

impl Default for RunMode {
    fn default() -> Self {
        Self::Loop { wait: None }
    }
}

/// The configuration information for the [`ScheduleRunnerPlugin`].
///
/// It gets added as a [`Resource`](lgn_ecs::system::Resource) inside of the [`ScheduleRunnerPlugin`].
#[derive(Copy, Clone, Default)]
pub struct ScheduleRunnerSettings {
    /// Determines whether the [`Schedule`](lgn_ecs::schedule::Schedule) is run once or repeatedly.
    pub run_mode: RunMode,
}

impl ScheduleRunnerSettings {
    /// See [`RunMode::Once`].
    pub fn run_once() -> Self {
        Self {
            run_mode: RunMode::Once,
        }
    }

    /// See [`RunMode::Loop`].
    pub fn run_loop(wait_duration: Duration) -> Self {
        Self {
            run_mode: RunMode::Loop {
                wait: Some(wait_duration),
            },
        }
    }
}

fn set_time_period() {
    // Windows is quantum for sleep can be set process wide and to a minimum of 1ms
    // even though it's set, depending on the windows version the value can be overridden
    // https://docs.microsoft.com/en-us/windows/win32/api/timeapi/nf-timeapi-timebeginperiod
    #[cfg(windows)]
    #[allow(unsafe_code)]
    unsafe {
        use lgn_tracing::error;
        use windows::Win32::Media::timeBeginPeriod;
        use windows::Win32::Media::TIMERR_NOERROR;

        const SLEEP_QUANTUM_MS: u32 = 1;
        let result = timeBeginPeriod(SLEEP_QUANTUM_MS);
        if result != TIMERR_NOERROR {
            error!("timeBeginPeriod failed with error code {}", result);
        } else {
            info!("timeBeginPeriod set to {}ms", SLEEP_QUANTUM_MS);
        }
    }
}

/// Configures an App to run its [Schedule](lgn_ecs::schedule::Schedule)
/// according to a given [`RunMode`]
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
                    if wait.is_some() {
                        set_time_period();
                    }

                    static CTRL_C_HIT: AtomicBool = AtomicBool::new(false);
                    ctrlc::set_handler(move || {
                        info!("Ctrl+C was hit!");
                        if CTRL_C_HIT.load(Ordering::SeqCst) {
                            std::process::exit(0);
                        }
                        CTRL_C_HIT.store(true, Ordering::SeqCst);
                    })
                    .expect("Error setting Ctrl+C handler");

                    let mut tick = move |app: &mut App,
                                         wait: Option<Duration>|
                          -> Result<Option<Duration>, AppExit> {
                        span_scope!("ScheduleRunnerPlugin::tick");
                        let start_time = Instant::now();

                        if let Some(mut app_exit_events) =
                            app.world.get_resource_mut::<Events<AppExit>>()
                        {
                            if let Some(exit) = app_exit_event_reader.iter(&app_exit_events).last()
                            {
                                return Err(exit.clone());
                            }
                            // give a chance for the app to handle the event during an update
                            // in case there is system reacting to the event and doing some cleanup
                            if CTRL_C_HIT.load(Ordering::SeqCst) {
                                app_exit_events.send(AppExit);
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

                        let elapsed = Instant::now() - start_time;
                        imetric!(
                            "Schedule Runner Tick Time",
                            "us",
                            elapsed.as_micros() as u64
                        );
                        if let Some(wait) = wait {
                            if elapsed < wait {
                                return Ok(Some(wait - elapsed));
                            }
                        }

                        Ok(None)
                    };

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        while let Ok(delay) = tick(&mut app, wait) {
                            if let Some(delay) = delay {
                                span_scope!("sleep");
                                std::thread::sleep(delay);
                            }
                        }
                    }

                    #[cfg(target_arch = "wasm32")]
                    {
                        fn set_timeout(f: &Closure<dyn FnMut()>, dur: Duration) {
                            web_sys::window()
                                .unwrap()
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    f.as_ref().unchecked_ref(),
                                    dur.as_millis() as i32,
                                )
                                .expect("Should register `setTimeout`.");
                        }
                        let asap = Duration::from_millis(1);

                        let mut rc = Rc::new(app);
                        let f = Rc::new(RefCell::new(None));
                        let g = f.clone();

                        let c = move || {
                            let mut app = Rc::get_mut(&mut rc).unwrap();
                            let delay = tick(&mut app, wait);
                            match delay {
                                Ok(delay) => {
                                    set_timeout(f.borrow().as_ref().unwrap(), delay.unwrap_or(asap))
                                }
                                Err(_) => {}
                            }
                        };
                        *g.borrow_mut() = Some(Closure::wrap(Box::new(c) as Box<dyn FnMut()>));
                        set_timeout(g.borrow().as_ref().unwrap(), asap);
                    };
                }
            }
        });
    }
}
