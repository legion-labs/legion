use http::Uri;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PingRequest {
    pub specified_uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerRequest {
    Ping(PingRequest),
}

impl ServerRequest {
    pub fn to_json(&self) -> Result<String, String> {
        match serde_json::to_string(&self) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Error formatting server request: {}", e)),
        }
    }
}

pub fn execute_request(server_uri: &str, request: &ServerRequest) -> Result<String, String> {
    let specified_uri = server_uri.parse::<Uri>().unwrap();
    let host = specified_uri.host().unwrap();
    let port = specified_uri.port_u16().unwrap_or(80);
    let url = format!("http://{}:{}/lsc", host, port);
    let client = reqwest::blocking::Client::new();
    match client.get(&url).body(request.to_json()?).send() {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                return Err(format!("Request {} failed with status {}", url, status));
            }
            match resp.text() {
                Ok(body) => Ok(body),
                Err(e) => Err(format!("Error reading response body: {}", e)),
            }
        }
        Err(e) => Err(format!("Error contacting server: {}", e)),
    }
}
