use crate::{
    server_request::{execute_request, PingRequest, ServerRequest},
    RepositoryUrl,
};
use anyhow::Result;
use lgn_tracing::span_fn;

#[span_fn]
pub async fn ping(repo_url: &RepositoryUrl) -> Result<()> {
    match repo_url {
        RepositoryUrl::Local(_) | RepositoryUrl::MySQL(_) => {}
        RepositoryUrl::Lsc(url) => {
            let client = reqwest::Client::new();
            let mut url = url.clone();
            url.set_path("");
            let request = ServerRequest::Ping(PingRequest {
                specified_uri: url.to_string(),
            });

            execute_request(&client, &url, &request).await?;
        }
    }

    Ok(())
}
