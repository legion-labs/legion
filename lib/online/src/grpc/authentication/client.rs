use std::future::Future;
use std::task::{Context, Poll};

use http::{Request, Response};
use http_body::{combinators::UnsyncBoxBody, Body};
use tonic::codegen::{BoxFuture, StdError};
use tower::Service;

use crate::authentication::Authenticator;

use super::super::buf::BoxBuf;
use super::{Error, Result};

/// A `gRPC` client wrapper that adds authentication.
#[derive(Clone)]
pub struct AuthenticatedClient<C, A> {
    client: C,
    authenticator: A,
}

impl<C, A> AuthenticatedClient<C, A>
where
    A: Authenticator + Clone,
{
    pub fn new(client: C, authenticator: A) -> Self {
        Self {
            client,
            authenticator,
        }
    }

    fn authenticate_request<'r, 'a, ReqBody>(
        &self,
        mut req: Request<ReqBody>,
    ) -> impl Future<Output = Result<Request<ReqBody>>> + 'a
    where
        ReqBody: 'r,
        A: 'a,
        'r: 'a,
    {
        let authenticator = self.authenticator.clone();

        async move {
            let token_set = authenticator
                .login()
                .await
                .map_err(Error::AuthenticationError)?;

            req.headers_mut().insert(
                "Authorization",
                format!("Bearer {}", token_set.access_token)
                    .parse()
                    .unwrap(),
            );

            Ok(req)
        }
    }
}

impl<C, A, ReqBody, ResBody> Service<Request<ReqBody>> for AuthenticatedClient<C, A>
where
    C: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + Sync + 'static,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    A: Authenticator + Clone + Send + Sync + 'static,
    ReqBody: Send + 'static,
    ResBody: http_body::Body + Send + 'static,
    <ResBody as http_body::Body>::Error: Into<StdError> + Send,
    <ResBody as http_body::Body>::Data: Send + 'static,
{
    type Response = Response<UnsyncBoxBody<BoxBuf, Error>>;
    type Error = Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.client
            .poll_ready(cx)
            .map_err(Into::into)
            .map_err(Error::Other)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let auth_req = self.authenticate_request(req);
        let mut client = self.client.clone();

        Box::pin(async move {
            let req = auth_req.await?;

            client
                .call(req)
                .await
                .map_err(Into::into)
                .map_err(Error::Other)
                .map(|resp| {
                    resp.map(|body| {
                        UnsyncBoxBody::new(
                            body.map_data(BoxBuf::new)
                                .map_err(Into::into)
                                .map_err(Error::Other),
                        )
                    })
                })
        })
    }
}
