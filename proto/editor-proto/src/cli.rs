use editor_client::EditorClient;

use std::io::{self, BufRead};

tonic::include_proto!("editor");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EditorClient::connect("http://[::1]:50051").await?;

    println!("Please enter the RTC session description:");

    let stdin = io::stdin();
    let rtc_session_description = base64::decode(stdin.lock().lines().next().unwrap().unwrap())?;

    let request = tonic::Request::new(InitializeStreamRequest {
        rtc_session_description,
    });

    let response = client.initialize_stream(request).await?.into_inner();

    if response.error.is_empty() {
        println!(
            "Stream initialized: {}",
            base64::encode(response.rtc_session_description),
        );
    } else {
        println!("Failed to initialize stream: {}", response.error,);
    }

    Ok(())
}
