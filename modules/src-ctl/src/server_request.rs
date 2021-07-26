use http::Uri;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PingRequest {
    pub specified_uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitRepositoryRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerRequest {
    Ping(PingRequest),
    InitRepo(InitRepositoryRequest),
}

impl ServerRequest {
    pub fn to_json(&self) -> Result<String, String> {
        match serde_json::to_string(&self) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Error formatting server request: {}", e)),
        }
    }

    pub fn from_json(contents: &str) -> Result<Self, String> {
        let parsed: serde_json::Result<Self> = serde_json::from_str(contents);
        match parsed {
            Ok(req) => Ok(req),
            Err(e) => Err(format!("Error parsing server request: {}", e)),
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
                return Err(format!(
                    "Request {} failed with status {}\n{}",
                    url,
                    status,
                    resp.text().unwrap_or_default()
                ));
            }
            match resp.text() {
                Ok(body) => Ok(body),
                Err(e) => Err(format!("Error reading response body: {}", e)),
            }
        }
        Err(e) => Err(format!("Error contacting server: {}", e)),
    }
}
