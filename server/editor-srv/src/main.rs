use std::time::Duration;

use legion_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use legion_async::{AsyncOperation, AsyncPlugin, TokioAsyncRuntime};
use legion_ecs::prelude::*;

struct Dropper {}

impl Drop for Dropper {
    fn drop(&mut self) {
        println!("I was dropped.")
    }
}

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(AsyncPlugin {})
        .add_startup_system(
            |mut commands: Commands, mut rt: ResMut<TokioAsyncRuntime>| {
                commands.spawn().insert(Salesman {
                    get_price: rt.start(async {
                        println!("Sleeping for one second...");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        42
                    }),
                    get_other_price: rt.start(async {
                        let _ = &Dropper {};
                        println!("Sleeping for three seconds...");
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        1234
                    }),
                });
            },
        )
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

struct Salesman {
    get_price: AsyncOperation<u32>,
    get_other_price: AsyncOperation<u32>,
}

fn online_loop_example(callers: Query<&Salesman>) {
    for caller in callers.iter() {
        if let Some(v) = caller.get_price.take_result() {
            match v {
                Ok(v) => {
                    println!("The price is: {:?}", v);
                    caller.get_other_price.cancel();
                }
                Err(e) => println!("Could not fetch the price: {}", e),
            };
        }
        if let Some(v) = caller.get_other_price.take_result() {
            match v {
                Ok(v) => println!("The other price is: {:?}", v),
                Err(e) => println!("Could not fetch the other price: {}", e),
            };
        }
    }
}

#[derive(Default)]
struct CounterState {
    count: u32,
}
