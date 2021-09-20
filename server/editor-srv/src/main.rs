use std::time::Duration;

use clap::Arg;
use legion_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use legion_async::{AsyncPlugin, TokioAsyncRuntime};
use legion_ecs::prelude::*;

mod server;
mod webrtc;

use server::Server;

fn main() {
    let args = clap::App::new("Legion Labs editor server")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about("Editor server.")
        .arg(
            Arg::with_name("addr")
                .long("addr")
                .takes_value(true)
                .help("The address to listen on"),
        )
        .get_matches();

    let addr = args
        .value_of("addr")
        .unwrap_or("[::1]:50051")
        .parse()
        .unwrap();

    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(AsyncPlugin {})
        .add_startup_system(move |rt: Res<TokioAsyncRuntime>| {
            let editor_server = Server::new().unwrap();
            println!("Starting editor server on: {}...", addr);

            rt.start_detached(Server::listen_and_serve(addr, editor_server));
        })
        .run();
}
