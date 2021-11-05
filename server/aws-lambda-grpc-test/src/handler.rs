use anyhow::{anyhow, Context as _};
use bytes::Bytes;
use http_body::Body;
use lambda_http::{IntoResponse, Request as LambdaRequest, Response as LambdaResponse};
use lambda_runtime::{Context, Error};

use legion_streaming_proto::{
    streamer_server::{Streamer, StreamerServer},
    InitializeStreamRequest, InitializeStreamResponse,
};
use log::info;
use tonic::{codegen::Service, Request, Response, Status};

struct MyStreamer;

pub async fn endpoint(
    request: LambdaRequest,
    _context: Context,
) -> Result<impl IntoResponse, Error> {
    // TODO: This should be done only once at the very beginning of the application.
    let mut server = StreamerServer::new(MyStreamer);

    // Take a lambda request and convert it to an HttpRequest that we can the feed to the `gRPC`
    // server.
    let tonic_request = into_tonic_request(request);
    let tonic_response = server.call(tonic_request).await?;
    from_tonic_response(tonic_response).await
}

fn into_tonic_request(request: LambdaRequest) -> http::Request<http_body::Full<Bytes>> {
    let (parts, body) = request.into_parts();
    let body = Bytes::from(body.to_vec());
    let body = http_body::Full::new(body);
    http::Request::from_parts(parts, body)

    //*result.version_mut() = http::Version::HTTP_2;
    //*result.method_mut() = http::Method::POST;
    //*result.uri_mut() = request.uri().to_owned();
    //*result.headers_mut() = *request.headers();
    //*request.extensions_mut() = self.extensions.into_http();
}

async fn from_tonic_response(
    response: lambda_http::Response<
        http_body::combinators::UnsyncBoxBody<bytes::Bytes, tonic::Status>,
    >,
) -> Result<impl IntoResponse, Error> {
    let (parts, mut body) = response.into_parts();
    let size_hint = body
        .size_hint()
        .exact()
        .ok_or_else(|| anyhow!("body size is not known"))?;
    let mut buf = Vec::<u8>::with_capacity(size_hint as usize);
    while let Some(chunk) = body.data().await {
        let chunk = chunk.context("failed to read data chunk")?;
        buf.extend_from_slice(&chunk);
    }
    let body = buf.to_vec();
    Ok(LambdaResponse::from_parts(parts, body))
}

#[tonic::async_trait]
impl Streamer for MyStreamer {
    async fn initialize_stream(
        &self,
        request: Request<InitializeStreamRequest>,
    ) -> Result<Response<InitializeStreamResponse>, Status> {
        let request = request.into_inner();
        info!("gRPC request received: {:?}", request);
        let response = InitializeStreamResponse {
            rtc_session_description: request.rtc_session_description,
            error: "".to_string(),
        };
        Ok(Response::new(response))
    }
}
