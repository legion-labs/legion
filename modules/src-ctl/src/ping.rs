use crate::server_request::*;

pub async fn ping_console_command(server_uri: &str) -> Result<(), String> {
    let request = ServerRequest::Ping(PingRequest {
        specified_uri: String::from(server_uri),
    });

    let client = reqwest::Client::new();
    let resp = execute_request(&client, server_uri, &request).await?;
    println!("{}", resp);
    Ok(())
}
