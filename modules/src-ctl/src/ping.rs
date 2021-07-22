use http::Uri;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PingCommand {
    pub specified_uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerCommand {
    Ping(PingCommand),
}

pub fn ping_console_command(server_uri: &str) -> Result<(), String> {
    let specified_uri = server_uri.parse::<Uri>().unwrap();
    let host = specified_uri.host().unwrap();
    let port = specified_uri.port_u16().unwrap_or(80);
    let url = format!("http://{}:{}/lsc", host, port);
    let command = ServerCommand::Ping(PingCommand {
        specified_uri: String::from(server_uri),
    });

    let client = reqwest::blocking::Client::new();
    match client
        .get(&url)
        .body(serde_json::to_string(&command).unwrap())
        .send()
    {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                return Err(format!("Request {} failed with status {}", url, status));
            }
            match resp.text() {
                Ok(body) => {
                    println!("{}", body);
                }
                Err(e) => {
                    return Err(format!("Error reading response body: {}", e));
                }
            }
        }
        Err(e) => {
            return Err(format!("Error contacting server: {}", e));
        }
    }
    Ok(())
}
