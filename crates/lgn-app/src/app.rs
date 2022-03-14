use std::fmt::Debug;

pub use lgn_derive::AppLabel;
use lgn_ecs::{
    prelude::{FromWorld, IntoExclusiveSystem},
    schedule::{
        IntoSystemDescriptor, RunOnce, Schedule, Stage, StageLabel, State, StateData, SystemSet,
        SystemStage,
    },
    system::Resource,
    world::World,
};
use lgn_telemetry_sink::TelemetryGuard;
use lgn_tracing::{debug, span_fn};
use lgn_utils::HashMap;

use crate::{
    CoreStage, Events, Plugin, PluginGroup, PluginGroupBuilder, StartupSchedule, StartupStage,
};

lgn_utils::define_label!(AppLabel);

#[allow(clippy::needless_doctest_main)]
/// Containers of app logic and data
///
/// Bundles together the necessary elements, like [`World`] and [`Schedule`], to create
/// an ECS-based application. It also stores a pointer to a
/// [runner function](Self::set_runner). The runner is responsible for managing the application's
/// event loop and applying the [`Schedule`] to the [`World`] to drive application logic.
/// Apps are constructed with the builder pattern.
///
/// ## Example
/// Here is a simple "Hello World" Legion app:
/// ```
/// # use lgn_app::prelude::*;
/// # use lgn_ecs::prelude::*;
///
/// fn main() {
///    App::default()
///        .add_system(hello_world_system)
///        .run();
/// }
///
/// fn hello_world_system() {
///    println!("hello world");
/// }
/// ```
pub struct App {
    /// The main ECS [`World`] of the [`App`].
    /// This stores and provides access to all the main data of the application.
    /// The systems of the [`App`] will run using this [`World`].
    /// If additional separate [`World`]-[`Schedule`] pairs are needed, you can use [`sub_app`][App::add_sub_app]s.
    pub world: World,
    /// The [runner function](Self::set_runner) is primarily responsible for managing
    /// the application's event loop and advancing the [`Schedule`].
    /// Typically, it is not configured manually, but set by one of Legion's built-in plugins.
    /// See `legion::winit::WinitPlugin` and [`ScheduleRunnerPlugin`](crate::schedule_runner::ScheduleRunnerPlugin).
    pub runner: Box<dyn FnOnce(App)>,
    /// A container of [`Stage`]s set to be run in a linear order.
    pub schedule: Schedule,
    sub_apps: HashMap<Box<dyn AppLabel>, SubApp>,
    telemetry_guard: Option<TelemetryGuard>,
}

/// Each [`SubApp`] has its own [`Schedule`] and [`World`], enabling a separation of concerns.
struct SubApp {
    app: App,
    runner: Box<dyn Fn(&mut World, &mut App)>,
}

impl Default for App {
    fn default() -> Self {
        Self::new(lgn_telemetry_sink::Config::default())
    }
}

impl App {
    /// Creates a new [`App`] with some default structure to enable core engine features.
    /// This is the preferred constructor for most use cases.
    pub fn new(telemetry_config: lgn_telemetry_sink::Config) -> Self {
        let mut app = Self::empty();

        app.telemetry_guard = Some(
            TelemetryGuard::new(telemetry_config)
                .expect("telemetry guard should be initialized once"),
        );

        app.add_default_stages()
            .add_event::<AppExit>()
            .add_system_to_stage(CoreStage::Last, World::clear_trackers.exclusive_system());

        #[cfg(feature = "lgn_ci_testing")]
        {
            crate::ci_testing::setup_app(&mut app);
        }

        app
    }

    /// Similar to [`App::new`] but takes a custom [`TelemetryGuard`] instead to allow for
    /// flexible telemetry setup.
    pub fn from_telemetry_guard(telemetry_guard: TelemetryGuard) -> Self {
        let mut app = Self::empty();

        app.telemetry_guard = Some(telemetry_guard);

        app.add_default_stages()
            .add_event::<AppExit>()
            .add_system_to_stage(CoreStage::Last, World::clear_trackers.exclusive_system());

        #[cfg(feature = "lgn_ci_testing")]
        {
            crate::ci_testing::setup_app(&mut app);
        }

        app
    }

    /// Creates a new empty [`App`] with minimal default configuration.
    ///
    /// This constructor should be used if you wish to provide a custom schedule, exit handling, cleanup, etc.
    pub fn empty() -> Self {
        Self {
            world: World::default(),
            schedule: Schedule::default(),
            runner: Box::new(run_once),
            sub_apps: HashMap::default(),
            telemetry_guard: None,
        }
    }

    /// Advances the execution of the [`Schedule`] by one cycle.
    ///
    /// This method also updates sub apps. See [`add_sub_app`](Self::add_sub_app) for more details.
    ///
    /// See [`Schedule::run_once`] for more details.
    #[span_fn]
    pub fn update(&mut self) {
        self.schedule.run(&mut self.world);
        for sub_app in self.sub_apps.values_mut() {
            (sub_app.runner)(&mut self.world, &mut sub_app.app);
        }
    }

    /// Starts the application by calling the app's [runner
    /// function](Self::set_runner).
    ///
    /// Finalizes the [`App`] configuration. For general usage, see the example
    /// on the item level documentation.
    #[span_fn]
    pub fn run(&mut self) {
        let mut app = std::mem::replace(self, Self::empty());
        let runner = std::mem::replace(&mut app.runner, Box::new(run_once));
        (runner)(app);
    }

    /// Adds a [`Stage`] with the given `label` to the last position of the
    /// app's [`Schedule`].
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// # let mut app = App::default();
    /// #
    /// app.add_stage("my_stage", SystemStage::parallel());
    /// ```
    pub fn add_stage<S: Stage>(&mut self, label: impl StageLabel, stage: S) -> &mut Self {
        self.schedule.add_stage(label, stage);
        self
    }

    /// Adds a [`Stage`] with the given `label` to the app's [`Schedule`],
    /// located immediately after the stage labeled by `target`.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// # let mut app = App::default();
    /// #
    /// app.add_stage_after(CoreStage::Update, "my_stage", SystemStage::parallel());
    /// ```
    pub fn add_stage_after<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        self.schedule.add_stage_after(target, label, stage);
        self
    }

    /// Adds a [`Stage`] with the given `label` to the app's [`Schedule`],
    /// located immediately before the stage labeled by `target`.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// # let mut app = App::default();
    /// #
    /// app.add_stage_before(CoreStage::Update, "my_stage", SystemStage::parallel());
    /// ```
    pub fn add_stage_before<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        self.schedule.add_stage_before(target, label, stage);
        self
    }

    /// Adds a [`Stage`] with the given `label` to the last position of the
    /// [startup schedule](Self::add_default_stages).
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// # let mut app = App::default();
    /// #
    /// app.add_startup_stage("my_startup_stage", SystemStage::parallel());
    /// ```
    pub fn add_startup_stage<S: Stage>(&mut self, label: impl StageLabel, stage: S) -> &mut Self {
        self.schedule
            .stage(StartupSchedule, |schedule: &mut Schedule| {
                schedule.add_stage(label, stage)
            });
        self
    }

    /// Adds a [startup stage](Self::add_default_stages) with the given `label`,
    /// immediately after the stage labeled by `target`.
    ///
    /// The `target` label must refer to a stage inside the startup schedule.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// # let mut app = App::default();
    /// #
    /// app.add_startup_stage_after(
    ///     StartupStage::Startup,
    ///     "my_startup_stage",
    ///     SystemStage::parallel()
    /// );
    /// ```
    pub fn add_startup_stage_after<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        self.schedule
            .stage(StartupSchedule, |schedule: &mut Schedule| {
                schedule.add_stage_after(target, label, stage)
            });
        self
    }

    /// Adds a [startup stage](Self::add_default_stages) with the given `label`,
    /// immediately before the stage labeled by `target`.
    ///
    /// The `target` label must refer to a stage inside the startup schedule.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// # let mut app = App::default();
    /// #
    /// app.add_startup_stage_before(
    ///     StartupStage::Startup,
    ///     "my_startup_stage",
    ///     SystemStage::parallel()
    /// );
    /// ```
    pub fn add_startup_stage_before<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        self.schedule
            .stage(StartupSchedule, |schedule: &mut Schedule| {
                schedule.add_stage_before(target, label, stage)
            });
        self
    }

    /// Fetches the [`Stage`] of type `T` marked with `label` from the
    /// [`Schedule`], then executes the provided `func` passing the fetched
    /// stage to it as an argument.
    ///
    /// The `func` argument should be a function or a closure that accepts a
    /// mutable reference to a struct implementing `Stage` and returns the
    /// same type. That means that it should also assume that the stage has
    /// already been fetched successfully.
    ///
    /// See [`Schedule::stage`] for more details.
    ///
    /// # Example
    ///
    /// Here the closure is used to add a system to the update stage:
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// #
    /// # let mut app = App::default();
    /// # fn my_system() {}
    /// #
    /// app.stage(CoreStage::Update, |stage: &mut SystemStage| {
    ///     stage.add_system(my_system)
    /// });
    /// ```
    pub fn stage<T: Stage, F: FnOnce(&mut T) -> &mut T>(
        &mut self,
        label: impl StageLabel,
        func: F,
    ) -> &mut Self {
        self.schedule.stage(label, func);
        self
    }

    /// Adds a system to the [update stage](Self::add_default_stages) of the
    /// app's [`Schedule`].
    ///
    /// Refer to the [system module documentation](lgn_ecs::system) to see how a
    /// system can be defined.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// #
    /// # fn my_system() {}
    /// # let mut app = App::default();
    /// #
    /// app.add_system(my_system);
    /// ```
    pub fn add_system<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut Self {
        self.add_system_to_stage(CoreStage::Update, system)
    }

    /// Adds a [`SystemSet`] to the [update stage](Self::add_default_stages).
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// #
    /// # let mut app = App::default();
    /// # fn system_a() {}
    /// # fn system_b() {}
    /// # fn system_c() {}
    /// #
    /// app.add_system_set(
    ///     SystemSet::new()
    ///         .with_system(system_a)
    ///         .with_system(system_b)
    ///         .with_system(system_c),
    /// );
    /// ```
    pub fn add_system_set(&mut self, system_set: SystemSet) -> &mut Self {
        self.add_system_set_to_stage(CoreStage::Update, system_set)
    }

    /// Adds a system to the [`Stage`] identified by `stage_label`.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// #
    /// # let mut app = App::default();
    /// # fn my_system() {}
    /// #
    /// app.add_system_to_stage(CoreStage::PostUpdate, my_system);
    /// ```
    pub fn add_system_to_stage<Params>(
        &mut self,
        stage_label: impl StageLabel,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        use std::any::TypeId;
        assert!(
            stage_label.type_id() != TypeId::of::<StartupStage>(),
            "add systems to a startup stage using App::add_startup_system_to_stage"
        );
        self.schedule.add_system_to_stage(stage_label, system);
        self
    }

    /// Adds a [`SystemSet`] to the [`Stage`] identified by `stage_label`.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// #
    /// # let mut app = App::default();
    /// # fn system_a() {}
    /// # fn system_b() {}
    /// # fn system_c() {}
    /// #
    /// app.add_system_set_to_stage(
    ///     CoreStage::PostUpdate,
    ///     SystemSet::new()
    ///         .with_system(system_a)
    ///         .with_system(system_b)
    ///         .with_system(system_c),
    /// );
    /// ```
    pub fn add_system_set_to_stage(
        &mut self,
        stage_label: impl StageLabel,
        system_set: SystemSet,
    ) -> &mut Self {
        use std::any::TypeId;
        assert!(
            stage_label.type_id() != TypeId::of::<StartupStage>(),
            "add system sets to a startup stage using App::add_startup_system_set_to_stage"
        );
        self.schedule
            .add_system_set_to_stage(stage_label, system_set);
        self
    }

    /// Adds a system to the [startup stage](Self::add_default_stages) of the
    /// app's [`Schedule`].
    ///
    /// * For adding a system that runs for every frame, see
    ///   [`add_system`](Self::add_system).
    /// * For adding a system to specific stage, see
    ///   [`add_system_to_stage`](Self::add_system_to_stage).
    ///
    /// ## Example
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// #
    /// fn my_startup_system(_commands: Commands) {
    ///     println!("My startup system");
    /// }
    ///
    /// App::default()
    ///     .add_startup_system(my_startup_system);
    /// ```
    pub fn add_startup_system<Params>(
        &mut self,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.add_startup_system_to_stage(StartupStage::Startup, system)
    }

    /// Adds a [`SystemSet`] to the [startup stage](Self::add_default_stages)
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// #
    /// # let mut app = App::default();
    /// # fn startup_system_a() {}
    /// # fn startup_system_b() {}
    /// # fn startup_system_c() {}
    /// #
    /// app.add_startup_system_set(
    ///     SystemSet::new()
    ///         .with_system(startup_system_a)
    ///         .with_system(startup_system_b)
    ///         .with_system(startup_system_c),
    /// );
    /// ```
    pub fn add_startup_system_set(&mut self, system_set: SystemSet) -> &mut Self {
        self.add_startup_system_set_to_stage(StartupStage::Startup, system_set)
    }

    /// Adds a system to the [startup schedule](Self::add_default_stages), in
    /// the stage identified by `stage_label`.
    ///
    /// `stage_label` must refer to a stage inside the startup schedule.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// #
    /// # let mut app = App::default();
    /// # fn my_startup_system() {}
    /// #
    /// app.add_startup_system_to_stage(StartupStage::PreStartup, my_startup_system);
    /// ```
    pub fn add_startup_system_to_stage<Params>(
        &mut self,
        stage_label: impl StageLabel,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.schedule
            .stage(StartupSchedule, |schedule: &mut Schedule| {
                schedule.add_system_to_stage(stage_label, system)
            });
        self
    }

    /// Adds a [`SystemSet`] to the [startup
    /// schedule](Self::add_default_stages), in the stage identified by
    /// `stage_label`.
    ///
    /// `stage_label` must refer to a stage inside the startup schedule.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// #
    /// # let mut app = App::default();
    /// # fn startup_system_a() {}
    /// # fn startup_system_b() {}
    /// # fn startup_system_c() {}
    /// #
    /// app.add_startup_system_set_to_stage(
    ///     StartupStage::PreStartup,
    ///     SystemSet::new()
    ///         .with_system(startup_system_a)
    ///         .with_system(startup_system_b)
    ///         .with_system(startup_system_c),
    /// );
    /// ```
    pub fn add_startup_system_set_to_stage(
        &mut self,
        stage_label: impl StageLabel,
        system_set: SystemSet,
    ) -> &mut Self {
        self.schedule
            .stage(StartupSchedule, |schedule: &mut Schedule| {
                schedule.add_system_set_to_stage(stage_label, system_set)
            });
        self
    }

    /// Adds a new [State] with the given `initial` value.
    /// This inserts a new `State<T>` resource and adds a new "driver" to
    /// [`CoreStage::Update`]. Each stage that uses `State<T>` for system
    /// run criteria needs a driver. If you need to use your state in a
    /// different stage, consider using [`Self::add_state_to_stage`] or manually
    /// adding [`State::get_driver`] to additional stages you need it in.
    pub fn add_state<T>(&mut self, initial: T) -> &mut Self
    where
        T: StateData,
    {
        self.add_state_to_stage(CoreStage::Update, initial)
    }

    /// Adds a new [State] with the given `initial` value.
    /// This inserts a new `State<T>` resource and adds a new "driver" to the
    /// given stage. Each stage that uses `State<T>` for system run criteria
    /// needs a driver. If you need to use your state in more than one
    /// stage, consider manually adding [`State::get_driver`] to the
    /// stages you need it in.
    pub fn add_state_to_stage<T>(&mut self, stage: impl StageLabel, initial: T) -> &mut Self
    where
        T: StateData,
    {
        self.insert_resource(State::new(initial))
            .add_system_set_to_stage(stage, State::<T>::get_driver())
    }

    /// Adds utility stages to the [`Schedule`], giving it a standardized
    /// structure.
    ///
    /// Adding those stages is necessary to make some core engine features work,
    /// like adding systems without specifying a stage, or registering
    /// events. This is however done by default by calling `App::default`,
    /// which is in turn called by [`App::new`].
    ///
    /// # The stages
    ///
    /// All the added stages, with the exception of the startup stage, run every
    /// time the schedule is invoked. The stages are the following, in order
    /// of execution:
    /// - **First:** Runs at the very start of the schedule execution cycle,
    ///   even before the startup stage.
    /// - **Startup:** This is actually a schedule containing sub-stages. Runs
    ///   only once when the app starts.
    ///     - **Pre-startup:** Intended for systems that need to run before
    ///       other startup systems.
    ///     - **Startup:** The main startup stage. Startup systems are added
    ///       here by default.
    ///     - **Post-startup:** Intended for systems that need to run after
    ///       other startup systems.
    /// - **Pre-update:** Often used by plugins to prepare their internal state
    ///   before the update stage begins.
    /// - **Update:** Intended for user defined logic. Systems are added here by
    ///   default.
    /// - **Post-update:** Often used by plugins to finalize their internal
    ///   state after the world changes that happened during the update stage.
    /// - **Last:** Runs right before the end of the schedule execution cycle.
    ///
    /// The labels for those stages are defined in the [`CoreStage`] and
    /// [`StartupStage`] `enum`s.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// #
    /// let app = App::empty().add_default_stages();
    /// ```
    pub fn add_default_stages(&mut self) -> &mut Self {
        self.add_stage(CoreStage::First, SystemStage::parallel())
            .add_stage(
                StartupSchedule,
                Schedule::default()
                    .with_run_criteria(RunOnce::default())
                    .with_stage(StartupStage::PreStartup, SystemStage::parallel())
                    .with_stage(StartupStage::Startup, SystemStage::parallel())
                    .with_stage(StartupStage::PostStartup, SystemStage::parallel()),
            )
            .add_stage(CoreStage::PreUpdate, SystemStage::parallel())
            .add_stage(CoreStage::Update, SystemStage::parallel())
            .add_stage(CoreStage::PostUpdate, SystemStage::parallel())
            .add_stage(CoreStage::Last, SystemStage::parallel())
    }

    /// Setup the application to manage events of type `T`.
    ///
    /// This is done by adding a `Resource` of type `Events::<T>`,
    /// and inserting a `Events::<T>::update_system` system into
    /// `CoreStage::First`.
    ///
    /// See [`Events`](lgn_ecs::event::Events) for defining events.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_app::prelude::*;
    /// # use lgn_ecs::prelude::*;
    /// #
    /// # struct MyEvent;
    /// # let mut app = App::default();
    /// #
    /// app.add_event::<MyEvent>();
    /// ```
    pub fn add_event<T>(&mut self) -> &mut Self
    where
        T: Resource,
    {
        self.init_resource::<Events<T>>()
            .add_system_to_stage(CoreStage::First, Events::<T>::update_system)
    }

    /// Inserts a resource to the current [App] and overwrites any resource
    /// previously added of the same type.
    ///
    /// A resource in Legion represents globally unique data. Resources must be
    /// added to Legion Apps before using them. This happens with
    /// [`insert_resource`](Self::insert_resource).
    ///
    /// See also `init_resource` for resources that implement `Default` or
    /// [`FromWorld`].
    ///
    /// ## Example
    /// ```
    /// # use lgn_app::prelude::*;
    /// #
    /// struct MyCounter {
    ///     counter: usize,
    /// }
    ///
    /// App::default()
    ///    .insert_resource(MyCounter { counter: 0 });
    /// ```
    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    /// Inserts a non-send resource to the app
    ///
    /// You usually want to use `insert_resource`,
    /// but there are some special cases when a resource cannot be sent across threads.
    ///
    /// ## Example
    /// ```
    /// # use lgn_app::prelude::*;
    /// #
    /// struct MyCounter {
    ///     counter: usize,
    /// }
    ///
    /// App::default()
    ///     .insert_non_send_resource(MyCounter { counter: 0 });
    /// ```
    pub fn insert_non_send_resource<R: 'static>(&mut self, resource: R) -> &mut Self {
        self.world.insert_non_send_resource(resource);
        self
    }

    /// Initialize a resource with standard starting values by adding it to the [`World`]
    ///
    /// If the resource already exists, nothing happens.
    ///
    /// The resource must implement the [`FromWorld`] trait.
    /// If the `Default` trait is implemented, the `FromWorld` trait will use
    /// the `Default::default` method to initialize the resource.
    ///
    /// ## Example
    /// ```
    /// # use lgn_app::prelude::*;
    /// #
    /// struct MyCounter {
    ///     counter: usize,
    /// }
    ///
    /// impl Default for MyCounter {
    ///     fn default() -> MyCounter {
    ///         MyCounter {
    ///             counter: 100
    ///         }
    ///     }
    /// }
    ///
    /// App::default()
    ///     .init_resource::<MyCounter>();
    /// ```
    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    /// Initialize a non-send resource with standard starting values by adding it to the [`World`]
    ///
    /// The resource must implement the [`FromWorld`] trait.
    /// If the `Default` trait is implemented, the `FromWorld` trait will use
    /// the `Default::default` method to initialize the resource.
    pub fn init_non_send_resource<R: 'static + FromWorld>(&mut self) -> &mut Self {
        self.world.init_non_send_resource::<R>();
        self
    }

    /// Sets the function that will be called when the app is run.
    ///
    /// The runner function (`run_fn`) is called only once by [`App::run`]. If
    /// the presence of a main loop in the app is desired, it is
    /// responsibility of the runner function to provide it.
    ///
    /// The runner function is usually not set manually, but by Legion
    /// integrated plugins (e.g. winit plugin).
    ///
    /// ## Example
    /// ```
    /// # use lgn_app::prelude::*;
    /// #
    /// fn my_runner(mut app: App) {
    ///     loop {
    ///         println!("In main loop");
    ///         app.update();
    ///     }
    /// }
    ///
    /// App::default()
    ///     .set_runner(my_runner);
    /// ```
    pub fn set_runner(&mut self, run_fn: impl FnOnce(Self) + 'static) -> &mut Self {
        self.runner = Box::new(run_fn);
        self
    }

    /// Adds a single plugin
    ///
    /// One of Legion's core principles is modularity. All Legion engine
    /// features are implemented as plugins. This includes internal features
    /// like the renderer.
    ///
    /// Legion also provides a few sets of default plugins. See
    /// [`add_plugins`](Self::add_plugins).
    ///
    /// ## Example
    /// ```ignore
    /// # use lgn_app::prelude::*;
    /// #
    /// App::default().add_plugin(lgn_transform::TransformPlugin::default());
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    #[span_fn]
    pub fn add_plugin<T>(&mut self, plugin: T) -> &mut Self
    where
        T: Plugin,
    {
        debug!("added plugin: {}", plugin.name());
        plugin.build(self);
        self
    }

    /// Adds a group of plugins
    ///
    /// Legion plugins can be grouped into a set of plugins. Legion provides
    /// built-in `PluginGroups` that provide core engine functionality.
    ///
    /// The plugin groups available by default are `DefaultPlugins` and
    /// `MinimalPlugins`.
    ///
    /// ## Example
    /// ```
    /// # use lgn_app::{prelude::*, PluginGroupBuilder};
    /// #
    /// # struct MinimalPlugins;
    /// # impl PluginGroup for MinimalPlugins {
    /// #     fn build(&mut self, group: &mut PluginGroupBuilder){;}
    /// # }
    /// #
    /// App::default()
    ///     .add_plugins(MinimalPlugins);
    /// ```
    pub fn add_plugins<T: PluginGroup>(&mut self, mut group: T) -> &mut Self {
        let mut plugin_group_builder = PluginGroupBuilder::default();
        group.build(&mut plugin_group_builder);
        plugin_group_builder.finish(self);
        self
    }

    /// Adds a group of plugins with an initializer method
    ///
    /// Can be used to add a group of plugins, where the group is modified
    /// before insertion into Legion application. For example, you can add
    /// extra plugins at a specific place in the plugin group, or deactivate
    /// specific plugins while keeping the rest.
    ///
    /// ## Example
    /// ```ignore
    /// # use lgn_app::{prelude::*, PluginGroupBuilder};
    /// #
    /// # struct DefaultPlugins;
    /// # impl PluginGroup for DefaultPlugins {
    /// #     fn build(&mut self, group: &mut PluginGroupBuilder){
    /// #         group.add(lgn_transform::TransformPlugin::default());
    /// #     }
    /// # }
    /// #
    /// # struct MyOwnPlugin;
    /// # impl Plugin for MyOwnPlugin {
    /// #     fn build(&self, app: &mut App){;}
    /// # }
    /// #
    /// App::default()
    ///      .add_plugins_with(DefaultPlugins, |group| {
    ///             group.add_before::<lgn_transform::TransformPlugin, _>(MyOwnPlugin)
    ///         });
    /// ```
    #[span_fn]
    pub fn add_plugins_with<T, F>(&mut self, mut group: T, func: F) -> &mut Self
    where
        T: PluginGroup,
        F: FnOnce(&mut PluginGroupBuilder) -> &mut PluginGroupBuilder,
    {
        let mut plugin_group_builder = PluginGroupBuilder::default();
        group.build(&mut plugin_group_builder);
        func(&mut plugin_group_builder);
        plugin_group_builder.finish(self);
        self
    }

    /// Adds an `App` as a child of the current one.
    ///
    /// The provided function `f` is called by the [`update`](Self::update) method. The `World`
    /// parameter represents the main app world, while the `App` parameter is just a mutable
    /// reference to the sub app itself.
    pub fn add_sub_app(
        &mut self,
        label: impl AppLabel,
        app: Self,
        sub_app_runner: impl Fn(&mut World, &mut Self) + 'static,
    ) -> &mut Self {
        self.sub_apps.insert(
            Box::new(label),
            SubApp {
                app,
                runner: Box::new(sub_app_runner),
            },
        );
        self
    }

    /// Retrieves a "sub app" stored inside this [App]. This will panic if the sub app does not exist.
    pub fn sub_app_mut(&mut self, label: impl AppLabel) -> &mut Self {
        match self.get_sub_app_mut(label) {
            Ok(app) => app,
            Err(label) => panic!("Sub-App with label '{:?}' does not exist", label),
        }
    }

    /// Retrieves a "sub app" inside this [App] with the given label, if it exists. Otherwise returns
    /// an [Err] containing the given label.
    #[allow(clippy::missing_errors_doc)]
    pub fn get_sub_app_mut(&mut self, label: impl AppLabel) -> Result<&mut Self, impl AppLabel> {
        self.sub_apps
            .get_mut((&label) as &dyn AppLabel)
            .map(|sub_app| &mut sub_app.app)
            .ok_or(label)
    }

    /// Retrieves a "sub app" stored inside this [App]. This will panic if the sub app does not exist.
    pub fn sub_app(&self, label: impl AppLabel) -> &Self {
        match self.get_sub_app(label) {
            Ok(app) => app,
            Err(label) => panic!("Sub-App with label '{:?}' does not exist", label),
        }
    }

    /// Retrieves a "sub app" inside this [App] with the given label, if it exists. Otherwise returns
    /// an [Err] containing the given label.
    #[allow(clippy::missing_errors_doc)]
    pub fn get_sub_app(&self, label: impl AppLabel) -> Result<&Self, impl AppLabel> {
        self.sub_apps
            .get((&label) as &dyn AppLabel)
            .map(|sub_app| &sub_app.app)
            .ok_or(label)
    }
}

fn run_once(mut app: App) {
    app.update();
}

/// An event that indicates the app should exit. This will fully exit the app
/// process.
#[derive(Debug, Clone, Default)]
pub struct AppExit;
