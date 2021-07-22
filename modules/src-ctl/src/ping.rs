use http::Uri;

pub fn ping_command(server_uri: &str) -> Result<(), String> {
    let specified_uri = server_uri.parse::<Uri>().unwrap();
    let host = specified_uri.host().unwrap();
    let port = specified_uri.port_u16().unwrap_or(80);
    let url = format!("http://{}:{}/lsc", host, port);

    let client = reqwest::blocking::Client::new();
    match client
        .get(&url)
        .header("command", format!("ping {}", server_uri))
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
