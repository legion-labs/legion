use std::time::Duration;

use legion_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use legion_ecs::prelude::*;
use legion_online::{OnlinePlugin, Result, Runtime};

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(OnlinePlugin)
        .add_startup_system(|mut commands: Commands| {
            let (_, age) = Result::new();
            commands.spawn().insert(Caller { age });
        })
        .add_system(frame_counter)
        .add_system(online_loop_example)
        .run();
}

fn frame_counter(mut state: Local<'_, CounterState>) {
    if state.count % 60 == 0 {
        println!("{}", state.count / 60);
    }
    state.count += 1;
}

struct Caller {
    age: Result<u32>,
}

fn online_loop_example(rt: Res<Runtime>, mut callers: Query<&mut Caller>) {
    for mut caller in callers.iter_mut() {
        if !caller.age.is_set() {
            caller.age = rt.spawn(async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                42
            });
        } else {
            println!("age: {:?}", caller.age.get());
        }
    }
}

#[derive(Default)]
struct CounterState {
    count: u32,
}
