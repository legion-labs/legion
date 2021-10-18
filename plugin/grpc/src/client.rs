//! Provides client helper methods for making `gRPC` calls.

use std::time::Duration;

use tonic::codegen::BoxFuture;

fn prepare_grpc_request<T>(mut req: hyper::Request<T>, uri: &hyper::Uri) -> hyper::Request<T> {
    let uri = hyper::Uri::builder()
        .scheme(uri.scheme().unwrap().clone())
        .authority(uri.authority().unwrap().clone())
        .path_and_query(req.uri().path_and_query().unwrap().clone())
        .build()
        .unwrap();

    *req.uri_mut() = uri;

    req
}

/// A `gRPC` generic client.
pub struct Client<Connector, ReqBody> {
    client: hyper::Client<Connector, ReqBody>,
    uri: hyper::Uri,
}

impl<Connector, ReqBody> Client<Connector, ReqBody>
where
    Connector: hyper::client::connect::Connect + Clone,
    ReqBody: tonic::codegen::Body + Send,
    <ReqBody as tonic::codegen::Body>::Data: Send,
{
    pub fn new_from_connector<U: Into<hyper::Uri>>(uri: U, connector: Connector) -> Self {
        Self {
            client: hyper::Client::builder().http2_only(true).build(connector),
            uri: uri.into(),
        }
    }
}

impl<Connector, ReqBody> Clone for Client<Connector, ReqBody>
where
    Connector: hyper::client::connect::Connect + Clone,
    ReqBody: tonic::codegen::Body,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            uri: self.uri.clone(),
        }
    }
}

impl<Connector, ReqBody> tower::Service<http::Request<ReqBody>> for Client<Connector, ReqBody>
where
    Connector: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    ReqBody: tonic::codegen::Body + Send + 'static,
    <ReqBody as tonic::codegen::Body>::Data: Send,
    <ReqBody as tonic::codegen::Body>::Error:
        Into<Box<(dyn std::error::Error + Send + Sync + 'static)>>,
{
    type Response = http::Response<hyper::Body>;

    type Error = hyper::Error;

    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let req = prepare_grpc_request(req, &self.uri);
        Box::pin(self.client.request(req))
    }
}

impl<ReqBody> Client<hyper::client::HttpConnector, ReqBody>
where
    ReqBody: tonic::codegen::Body + Send,
    <ReqBody as tonic::codegen::Body>::Data: Send,
{
    pub fn new<U: Into<hyper::Uri>>(uri: U) -> Self {
        let mut connector = hyper::client::HttpConnector::new();
        connector.set_connect_timeout(Some(Duration::from_secs(5)));

        Self::new_from_connector(uri, connector)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_prepare_grpc_request() {
        let uri = hyper::Uri::from_static("http://127.0.0.1:50051");
        let mut req = hyper::Request::new(42);
        *req.uri_mut() = hyper::Uri::from_static(
            "https://user:password@host:port/foo/bar?query=string#fragment",
        );

        // The scheme, authority and path+query should be taken from the second argument.
        let req = prepare_grpc_request(req, &uri);

        assert_eq!(
            req.uri().to_string(),
            "http://127.0.0.1:50051/foo/bar?query=string"
        );
    }
}
