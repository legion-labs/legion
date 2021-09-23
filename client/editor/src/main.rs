#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::error::Error;
use tokio::sync::Mutex;

use legion_editor_proto::{editor_client::*, InitializeStreamRequest};
use tonic::transport::Channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let client = Mutex::new(EditorClient::connect("http://[::1]:50051").await?);

  tauri::Builder::default()
    .manage(client)
    .invoke_handler(tauri::generate_handler![initialize_stream])
    .run(tauri::generate_context!())?;

  Ok(())
}

#[tauri::command]
async fn initialize_stream(
  client: tauri::State<'_, Mutex<EditorClient<Channel>>>,
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
