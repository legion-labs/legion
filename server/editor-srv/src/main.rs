use std::time::Duration;

use clap::Arg;
use legion_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use legion_async::{AsyncPlugin, TokioAsyncRuntime};
use legion_ecs::prelude::*;
use legion_editor_proto::{editor_server::*, *};
use tonic::{Request, Response, Status};

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
            let editor_server = Server::default();

            println!("Starting editor server on: {}...", addr);

            let future = tonic::transport::Server::builder()
                .add_service(EditorServer::new(editor_server))
                .serve(addr);

            rt.start_detached(future);
        })
        .run();
}

#[derive(Debug, Default)]
pub struct Server {}

#[tonic::async_trait]
impl Editor for Server {
    async fn update_properties(
        &self,
        request: Request<UpdatePropertiesRequest>,
    ) -> Result<Response<UpdatePropertiesResponse>, Status> {
        let m = request.into_inner();

        println!(
            "property {} was updated ({}): {}",
            m.property_path, m.update_id, m.value
        );

        let response = UpdatePropertiesResponse {
            update_id: m.update_id,
        };

        Ok(Response::new(response))
    }
}
