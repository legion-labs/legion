mod errors;

use errors::{Error, Result};
use http::{Request, Response};
use hyper::service::Service;

/// A client for the governance service.
pub struct Client<Inner> {
    user_client: crate::api::user::client::Client<Inner>,
}

impl<Inner> Client<Inner> {
    /// Creates a new client.
    pub fn new(inner: Inner, base_uri: http::Uri) -> Self {
        Self {
            user_client: crate::api::user::client::Client::new(inner, base_uri),
        }
    }
}

impl<Inner, ResBody> Client<Inner>
where
    Inner: Service<Request<hyper::Body>, Response = Response<ResBody>> + Send + Sync + Clone,
    Inner::Error: Into<lgn_online::client::Error>,
    Inner::Future: Send,
    ResBody: hyper::body::HttpBody + Send,
    ResBody::Data: Send,
    ResBody::Error: std::error::Error,
{
    /// Initialize the stack.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions or if the stack was already initialized.
    pub async fn init_stack(&self, init_key: &str) -> Result<()> {
        use crate::api::user::client::{InitStackRequest, InitStackResponse};

        match self
            .user_client
            .init_stack(InitStackRequest {
                x_init_key: init_key.to_string(),
            })
            .await?
        {
            InitStackResponse::Status200 { .. } => Ok(()),
            InitStackResponse::Status403 { .. } => Err(Error::Unauthorized),
            InitStackResponse::Status409 { .. } => Err(Error::StackAlreadyInitialized),
        }
    }
}
