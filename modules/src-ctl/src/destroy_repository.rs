use crate::server_request::*;
use crate::sql;
use url::Url;

pub async fn destroy_repository_command(uri: &str) -> Result<(), String> {
    let repo_uri = Url::parse(uri).unwrap();
    match repo_uri.scheme() {
        "file" => {
            return Err(String::from(
                "file:// scheme not implemented, remove the directory manually",
            ));
        }
        "mysql" => {
            sql::drop_database(uri).await?;
        }
        "lsc" => {
            let mut url_path = String::from(repo_uri.path());
            let path = url_path.split_off(1); //remove leading /
            let request = ServerRequest::DestroyRepo(DestroyRepositoryRequest { repo_name: path });
            let host = repo_uri.host().unwrap();
            let port = repo_uri.port().unwrap_or(80);
            let url = format!("http://{}:{}/lsc", host, port);
            let client = reqwest::Client::new();
            let resp = execute_request(&client, &url, &request).await?;
            println!("{}", resp);
        }
        unknown => {
            return Err(format!("Unknown repository scheme {}", unknown));
        }
    }
    Ok(())
}
