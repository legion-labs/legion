use anyhow::Result;

use crate::server_request::{execute_request, DestroyRepositoryRequest, ServerRequest};
use crate::{sql, RepositoryUrl};

pub async fn destroy_repository(repo_url: &RepositoryUrl) -> Result<()> {
    lgn_telemetry::trace_scope!();

    match repo_url {
        RepositoryUrl::Local(_) => {
            anyhow::bail!("file:// scheme not implemented, remove the directory manually");
        }
        RepositoryUrl::MySQL(url) => {
            sql::drop_database(url.as_str()).await?;
        }
        RepositoryUrl::Lsc(url) => {
            let repo_name = String::from(url.path()).split_off(1); //remove leading /
            let client = reqwest::Client::new();
            let mut url = url.clone();
            url.set_path("");
            let request = ServerRequest::DestroyRepo(DestroyRepositoryRequest { repo_name });

            execute_request(&client, &url, &request).await?;
        }
    }

    Ok(())
}
