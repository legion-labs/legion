use std::sync::Arc;

use http::{Request, Response};
use lambda_http::Handler;
use tokio::sync::Mutex;
use tonic::{body::BoxBody, codegen::BoxFuture};
use tower::Service;

/// An AWS Lambda handler that implements the GRPC service.
pub struct AwsLambdaHandler<S> {
    inner: Arc<Mutex<S>>,
}

impl<S> AwsLambdaHandler<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

impl<'a, S> Handler<'a> for AwsLambdaHandler<S>
where
    S: Service<Request<hyper::Body>, Response = Response<BoxBody>> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<lambda_runtime::Error>,
{
    type Error = lambda_runtime::Error;
    type Response = Response<Vec<u8>>;
    type Fut = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, event: lambda_http::Request, _context: lambda_http::Context) -> Self::Fut {
        let request = event.map(|b| b.to_vec().into());
        let inner = Arc::clone(&self.inner);

        Box::pin(async move {
            let mut inner = inner.lock().await;
            let response = inner.call(request).await.map_err(Into::into)?;
            drop(inner);

            let (parts, body) = response.into_parts();
            let body = hyper::body::to_bytes(body).await?.to_vec();
            Ok(lambda_http::Response::from_parts(parts, body))
        })
    }
}
