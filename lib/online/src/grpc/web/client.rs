use std::task::{Context, Poll};

use http::{HeaderMap, HeaderValue, Method, Request, Response, Version};
use http_body::{combinators::UnsyncBoxBody, Body};
use log::{debug, warn};
use tonic::{
    body::BoxBody,
    client::GrpcService,
    codegen::{BoxFuture, StdError},
};

use super::super::consts::{GRPC, GRPC_WEB, PROTOBUF};

use super::{Error, Result};

/// A gRPC-Web client wrapper that causes all outgoing `gRPC` requests to be sent as HTTP 1.1
/// gRPC-Web requests.
#[derive(Clone)]
pub struct GrpcWebClient<C> {
    inner: C,
}

use super::{BoxBuf, GrpcWebResponse};

type RequestTransform = fn(Request<BoxBody>) -> Result<Request<BoxBody>>;
type ResponseTransform<F> = fn(F) -> BoxFuture<Response<UnsyncBoxBody<BoxBuf, Error>>, Error>;

impl<C> GrpcWebClient<C>
where
    C: GrpcService<BoxBody>,
    C::Future: Send + 'static,
    C::ResponseBody: Send + 'static,
    <C::ResponseBody as Body>::Data: Send,
    <C::ResponseBody as Body>::Error: Into<StdError>,
{
    pub fn new(c: C) -> Self {
        Self { inner: c }
    }

    fn forward_request(
        &mut self,
        request: Request<BoxBody>,
    ) -> BoxFuture<Response<UnsyncBoxBody<BoxBuf, Error>>, Error> {
        let resp = self.inner.call(request);

        Box::pin(async move {
            // This might look complex but we just wrap the response body data into a `BoxBuf`,
            // convert its error into an `Error`, and then wrap the response body into a
            // `UnsyncBoxBody` so that the end result satisfies our response type.
            //
            // This allows us to keep any streaming/trailers support and pass them through
            // untouched.
            Ok(resp.await.map_err(Into::into)?.map(|body| {
                UnsyncBoxBody::new(
                    body.map_data(BoxBuf::new)
                        .map_err(Into::into)
                        .map_err(Error::Other),
                )
            }))
        })
    }

    fn get_transform_functions(
        proto: &str,
    ) -> Result<(RequestTransform, ResponseTransform<C::Future>)> {
        match proto {
            PROTOBUF => Ok((
                Self::coerce_request_into_protobuf,
                Self::coerce_response_from_protobuf,
            )),
            _ => Err(Error::UnsupportedGrpcProtocol(proto.to_string())),
        }
    }

    fn transform_request(
        &mut self,
        request: Request<BoxBody>,
        proto: &str,
    ) -> BoxFuture<Response<UnsyncBoxBody<BoxBuf, Error>>, Error> {
        debug!("transforming gRPC call with protocol `{}`", proto);

        let (coerce_request, coerce_response) = match Self::get_transform_functions(proto) {
            Ok(f) => f,
            Err(e) => return Box::pin(async move { Err(e) }),
        };

        let resp = match coerce_request(request) {
            Ok(request) => self.inner.call(request),
            Err(err) => return Box::pin(async move { Err(err) }),
        };

        coerce_response(resp)
    }

    /// Transforms a `gRPC` HTTP 2 request into a `gRPC-Web` HTTP 1.1 request.
    ///
    /// We only support `application/grp-web+proto` as a content type for now.
    fn coerce_request_into_protobuf(mut req: Request<BoxBody>) -> Result<Request<BoxBody>> {
        let content_type = HeaderValue::from_str(&format!("{}+{}", GRPC_WEB, PROTOBUF))
            .map_err(Into::into)
            .map_err(Error::Other)?;

        // We must force HTTP 1.1 for `gRPC-web`.
        *req.version_mut() = http::Version::HTTP_11;

        req.headers_mut()
            .insert(http::header::CONTENT_TYPE, content_type.clone());
        req.headers_mut().insert(http::header::ACCEPT, content_type);

        // If `gRPC` clients were allowed to send trailers, this is where we would extend the
        // request and include them. Luckily for us, this is not supported so we don't have to
        // care.

        Ok(req)
    }

    /// Transforms a `gRPC`-web request into its HTTP 2 equivalent.
    ///
    /// This functions assumes that the request that caused the specified response was transformed
    /// with `coerce_request`.
    fn coerce_response_from_protobuf(
        resp: C::Future,
    ) -> BoxFuture<Response<UnsyncBoxBody<BoxBuf, Error>>, Error> {
        Box::pin(async move {
            let content_type = HeaderValue::from_str(&format!("{}+{}", GRPC, PROTOBUF))
                .map_err(Into::into)
                .map_err(Error::Other)?;

            let (mut parts, body) = resp.await.map_err(Into::into)?.into_parts();
            parts.version = http::Version::HTTP_2;

            parts
                .headers
                .insert(http::header::CONTENT_TYPE, content_type);

            let body = GrpcWebResponse::new(body);

            let resp = Response::from_parts(parts, UnsyncBoxBody::new(body));

            Ok(resp)
        })
    }
}

impl<C> GrpcService<BoxBody> for GrpcWebClient<C>
where
    C: GrpcService<BoxBody>,
    C::Error: Into<StdError>,
    C::Future: Send + 'static,
    C::ResponseBody: Send + 'static,
    <C::ResponseBody as Body>::Data: Send,
    <C::ResponseBody as Body>::Error: Into<StdError>,
{
    type ResponseBody = UnsyncBoxBody<BoxBuf, Error>;
    type Error = Error;
    type Future = BoxFuture<Response<Self::ResponseBody>, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.inner
            .poll_ready(cx)
            .map_err(Into::into)
            .map_err(Error::Other)
    }

    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        use RequestKind::{Grpc, Other};

        match RequestKind::new(request.headers(), request.method(), request.version()) {
            Grpc {
                method: &Method::POST,
                proto,
            } => self.transform_request(request, &proto),
            Grpc { method, proto } => {
                warn!(
                    "refusing to handle gRPC call with invalid method `{}` and protocol `{}`",
                    method, proto
                );

                self.forward_request(request)
            }
            Other(..) => self.forward_request(request),
        }
    }
}

#[derive(Debug, PartialEq)]
enum RequestKind<'a> {
    // The request is considered a grpc request if its `content-type`
    // header is "application/grpc" and the HTTP method is HTTP 2.
    Grpc { method: &'a Method, proto: String },
    // All other requests, including `application/grpc`
    Other(Version),
}

impl<'a> RequestKind<'a> {
    fn new(headers: &'a HeaderMap, method: &'a Method, version: Version) -> Self {
        match headers
            .get(http::header::CONTENT_TYPE)
            .and_then(|val| val.to_str().ok())
        {
            Some(content_type) => match content_type.strip_prefix(GRPC) {
                Some(proto) => {
                    let mut proto = proto.trim_start_matches('+');

                    if proto.is_empty() {
                        proto = PROTOBUF;
                    }

                    RequestKind::Grpc {
                        method,
                        proto: proto.to_string(),
                    }
                }
                None => RequestKind::Other(version),
            },
            None => RequestKind::Other(version),
        }
    }
}
