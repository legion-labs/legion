use std::time::Duration;

use legion_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use legion_ecs::prelude::*;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_system(frame_counter)
        .run();
}

fn frame_counter(mut state: Local<'_, CounterState>) {
    if state.count % 60 == 0 {
        println!("{}", state.count / 60);
    }
    state.count += 1;
}

#[derive(Default)]
struct CounterState {
    count: u32,
}
