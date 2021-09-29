#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use async_std::future::timeout;
use clap::Arg;
use legion_app::prelude::*;
use legion_async::AsyncPlugin;
use legion_editor_proto::{editor_client::*, InitializeStreamRequest};
use legion_tauri::legion_tauri_command;
use std::{cell::RefCell, error::Error, rc::Rc, time::Duration};
use tauri::Event;

struct Config {
    server_addr: String,
}

impl Config {
    fn new(args: clap::ArgMatches) -> anyhow::Result<Self> {
        Ok(Self {
            server_addr: args
                .value_of("server-addr")
                .unwrap_or("http://[::1]:50051")
                .parse()?,
        })
    }

    fn new_from_environment() -> anyhow::Result<Self> {
        let args = clap::App::new("Legion Labs editor")
            .author(clap::crate_authors!())
            .version(clap::crate_version!())
            .about("Legion Labs editor.")
            .arg(
                Arg::with_name("server-addr")
                    .long("server-addr")
                    .takes_value(true)
                    .help("The address of the editor server to connect to"),
            )
            .get_matches();

        Self::new(args)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new_from_environment()?;

    let tauri_app = tauri::Builder::default()
        .manage(config)
        .invoke_handler(tauri::generate_handler![initialize_stream])
        .build(tauri::generate_context!())
        .expect("failed to instanciate a Tauri App");

    App::new()
        .set_runner(TauriRunner::build(tauri_app))
        .add_plugin(AsyncPlugin {})
        .run();

    Ok(())
}

struct TauriRunner {}

impl TauriRunner {
    fn build(tauri_app: tauri::App<tauri::Wry>) -> impl FnOnce(App) {
        move |app: App| {
            // FIXME: Once https://github.com/tauri-apps/tauri/pull/2667 is merged, we can
            // get rid of this and move the value directly instead.
            let app = Rc::new(RefCell::new(app));

            tauri_app.run(move |_, event| {
                if let Event::MainEventsCleared = event {
                    app.borrow_mut().update();
                }
            });
        }
    }
}

#[legion_tauri_command]
async fn initialize_stream(
    config: tauri::State<'_, Config>,
    rtc_session_description: String,
) -> anyhow::Result<String> {
    let mut client = timeout(
        Duration::from_secs(3),
        EditorClient::connect(config.server_addr.clone()),
    )
    .await??;

    let rtc_session_description = base64::decode(rtc_session_description)?;
    let request = tonic::Request::new(InitializeStreamRequest {
        rtc_session_description,
    });

    let response = client.initialize_stream(request).await?.into_inner();

    if response.error.is_empty() {
        Ok(base64::encode(response.rtc_session_description))
    } else {
        Err(anyhow::format_err!("{}", response.error))
    }
}
