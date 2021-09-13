use std::time::Duration;

use legion_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use legion_ecs::prelude::*;
use legion_online::{OnlineOperation, OnlineOperationStatus::*, OnlinePlugin, TokioOnlineRuntime};

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(OnlinePlugin)
        .add_startup_system(|mut commands: Commands| {
            commands.spawn().insert(Caller {
                get_age: OnlineOperation::default(),
            });
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
    get_age: OnlineOperation<u32>,
}

fn online_loop_example(rt: Res<TokioOnlineRuntime>, mut callers: Query<&mut Caller>) {
    for mut caller in callers.iter_mut() {
        match caller.get_age.poll(rt.as_ref()) {
            Idle => {
                println!("idle");
                caller
                    .get_age
                    .start_with(rt.as_ref(), async {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        42
                    })
                    .unwrap();
            }
            Completed(v) => {
                println!("completed: {:?}", v);
                caller.get_age.restart_with(rt.as_ref(), async {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    43
                });
            }
            _ => {}
        };
    }
}

#[derive(Default)]
struct CounterState {
    count: u32,
}
