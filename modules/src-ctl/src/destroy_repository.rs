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
            sql::drop_database(uri)?;
        }
        "lsc" => {
            return Err(String::from("lsc scheme not implemented"));
        }
        unknown => {
            return Err(format!("Unknown repository scheme {}", unknown));
        }
    }
    Ok(())
}
