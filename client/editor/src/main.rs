#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use clap::Arg;
use legion_app::prelude::*;
use legion_async::AsyncPlugin;
use legion_editor_proto::{editor_client::*, InitializeStreamRequest};
use std::{
    cell::RefCell,
    error::Error,
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    rc::Rc,
};
use tauri::Event;
use tokio::sync::{Mutex, MutexGuard};
use tonic::transport::Channel;

fn main() -> Result<(), Box<dyn Error>> {
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

    let server_addr: String = args
        .value_of("server-addr")
        .unwrap_or("http://[::1]:50051")
        .parse()
        .unwrap();

    let client = LazyMutex::new(async move { EditorClient::connect(server_addr).await.unwrap() });

    let tauri_app = tauri::Builder::default()
        .manage(client)
        .invoke_handler(tauri::generate_handler![initialize_stream])
        .build(tauri::generate_context!())
        .expect("failed to instanciate a Tauri App");

    App::new()
        .set_runner(TauriRunner::build(tauri_app))
        .add_plugin(AsyncPlugin {})
        .run();

    Ok(())
}

struct LazyMutex<T> {
    value: Mutex<LazyMutexValue<T>>,
}

struct LazyMutexGuard<'a, T> {
    guard: MutexGuard<'a, LazyMutexValue<T>>,
}

impl<'a, T> Deref for LazyMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.value()
    }
}

impl<'a, T> DerefMut for LazyMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.value_mut()
    }
}

enum LazyMutexValue<T> {
    Future(Pin<Box<dyn Future<Output = T> + Send + 'static>>),
    Value(T),
}

impl<T> LazyMutexValue<T> {
    fn value(&self) -> &T {
        match self {
            LazyMutexValue::Value(v) => v,
            _ => {
                panic!("not a value");
            }
        }
    }

    fn value_mut(&mut self) -> &mut T {
        match self {
            LazyMutexValue::Value(v) => v,
            _ => {
                panic!("not a value");
            }
        }
    }
}

impl<T> LazyMutex<T> {
    fn new<F>(f: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Self {
            value: Mutex::new(LazyMutexValue::Future(Box::pin(f))),
        }
    }

    async fn lock(&self) -> LazyMutexGuard<'_, T> {
        let mut guard = self.value.lock().await;

        if let LazyMutexValue::Future(f) = guard.deref_mut() {
            *guard.deref_mut() = LazyMutexValue::Value(f.await);
        }

        LazyMutexGuard { guard }
    }
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

#[tauri::command]
async fn initialize_stream(
    client: tauri::State<'_, LazyMutex<EditorClient<Channel>>>,
    rtc_session_description: String,
) -> Result<String, String> {
    let mut client = client.lock().await;

    match initialize_stream_impl(&mut client, rtc_session_description).await {
        Ok(rtc_session_description) => Ok(rtc_session_description),
        Err(e) => Err(format!("{}", e)),
    }
}

async fn initialize_stream_impl(
    client: &mut EditorClient<Channel>,
    rtc_session_description: String,
) -> Result<String, Box<dyn Error>> {
    let rtc_session_description = base64::decode(rtc_session_description)?;
    let request = tonic::Request::new(InitializeStreamRequest {
        rtc_session_description,
    });

    let response = client.initialize_stream(request).await?.into_inner();

    if response.error.is_empty() {
        Ok(base64::encode(response.rtc_session_description))
    } else {
        Err(response.error.into())
    }
}
