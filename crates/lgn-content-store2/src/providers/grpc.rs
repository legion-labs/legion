use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures::{stream::TryStreamExt, Future};
use http::Uri;
use lgn_content_store_proto::{
    content_store_client::ContentStoreClient, read_content_response::Content,
};
use lgn_online::grpc::GrpcWebClient;
use pin_project::pin_project;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};

use crate::{ContentReader, ContentWriter, Error, Identifier, Result};

/// A `GrpcProvider` is a provider that delegates to a `gRPC` service.
pub struct GrpcProvider {
    client: Arc<Mutex<ContentStoreClient<GrpcWebClient>>>,
}

impl GrpcProvider {
    pub async fn new(uri: Uri) -> Self {
        let client = Arc::new(Mutex::new(ContentStoreClient::new(GrpcWebClient::new(uri))));

        Self { client }
    }
}

#[async_trait]
impl ContentReader for GrpcProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncRead + Send>>> {
        let req = lgn_content_store_proto::ReadContentRequest { id: id.to_string() };

        let resp = self
            .client
            .lock()
            .await
            .read_content(req)
            .await
            .map_err(|err| anyhow::anyhow!("gRPC request failed: {}", err))?
            .into_inner();

        match resp.content {
            Some(content) => Ok(match content {
                Content::Data(data) => Box::pin(std::io::Cursor::new(data)),
                Content::Url(url) => Box::pin(
                    reqwest::get(&url)
                        .await
                        .map_err(|err| {
                            anyhow::anyhow!("failed to fetch content from {}: {}", url, err)
                        })?
                        .error_for_status()
                        .map_err(|err| anyhow::anyhow!("HTTP error: {}", err))?
                        .bytes_stream()
                        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
                        .into_async_read()
                        .compat(),
                ),
            }),
            None => Err(Error::NotFound),
        }
    }
}

#[async_trait]
impl ContentWriter for GrpcProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncWrite + Send>>> {
        let req = lgn_content_store_proto::GetContentWriterRequest { id: id.to_string() };

        let resp = self
            .client
            .lock()
            .await
            .get_content_writer(req)
            .await
            .map_err(|err| anyhow::anyhow!("gRPC request failed: {}", err))?
            .into_inner();

        match resp.content_writer {
            Some(lgn_content_store_proto::get_content_writer_response::ContentWriter::Url(url)) => {
                if url.is_empty() {
                    Ok(Box::pin(GrpcUploader::new(
                        Arc::clone(&self.client),
                        id.clone(),
                    )))
                } else {
                    let uploader = HttpUploader::new(url)?;

                    Ok(Box::pin(uploader))
                }
            }
            None => Err(Error::AlreadyExists),
        }
    }
}

#[pin_project]
struct GrpcUploader {
    #[pin]
    state: GrpcUploaderState,
}

enum GrpcUploaderState {
    Invalid,
    Writing(
        std::io::Cursor<Vec<u8>>,
        Identifier,
        Arc<Mutex<ContentStoreClient<GrpcWebClient>>>,
    ),
    Uploading(Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'static>>),
}

impl GrpcUploader {
    pub fn new(client: Arc<Mutex<ContentStoreClient<GrpcWebClient>>>, id: Identifier) -> Self {
        let state = GrpcUploaderState::Writing(std::io::Cursor::new(Vec::new()), id, client);

        Self { state }
    }

    async fn upload(
        data: Vec<u8>,
        id: Identifier,
        client: Arc<Mutex<ContentStoreClient<GrpcWebClient>>>,
    ) -> Result<(), std::io::Error> {
        let req = lgn_content_store_proto::WriteContentRequest {
            id: id.to_string(),
            data,
        };

        client
            .lock()
            .await
            .write_content(req)
            .await
            .map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    anyhow::anyhow!("gRPC request failed: {}", err),
                )
            })?;

        Ok(())
    }
}

impl AsyncWrite for GrpcUploader {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        let this = self.project();

        if let GrpcUploaderState::Writing(cursor, _, _) = this.state.get_mut() {
            Pin::new(cursor).poll_write(cx, buf)
        } else {
            panic!("HttpUploader::poll_write called after completion")
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();

        if let GrpcUploaderState::Writing(cursor, _, _) = this.state.get_mut() {
            Pin::new(cursor).poll_flush(cx)
        } else {
            panic!("HttpUploader::poll_flush called after completion")
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();
        let state = this.state.get_mut();

        loop {
            *state = match std::mem::replace(state, GrpcUploaderState::Invalid) {
                GrpcUploaderState::Invalid => unreachable!("the state should never be invalid"),
                GrpcUploaderState::Writing(mut cursor, id, client) => {
                    match Pin::new(&mut cursor).poll_shutdown(cx) {
                        Poll::Ready(Ok(())) => GrpcUploaderState::Uploading(Box::pin(
                            Self::upload(cursor.into_inner(), id, client),
                        )),
                        p => return p,
                    }
                }
                GrpcUploaderState::Uploading(mut call) => return Pin::new(&mut call).poll(cx),
            };
        }
    }
}

#[pin_project]
struct HttpUploader {
    #[pin]
    w: Option<tokio_pipe::PipeWrite>,

    #[pin]
    call: Pin<Box<dyn Future<Output = Result<reqwest::Response, reqwest::Error>> + Send + 'static>>,
}

impl HttpUploader {
    pub fn new(url: String) -> Result<Self> {
        let client = reqwest::Client::new();

        let (r, w) =
            tokio_pipe::pipe().map_err(|err| anyhow::anyhow!("failed to create pipe: {}", err))?;

        let stream = ReaderStream::new(r);
        let w = Some(w);
        let call = Box::pin(
            client
                .post(url)
                .body(reqwest::Body::wrap_stream(stream))
                .send(),
        );

        Ok(Self { w, call })
    }
}

#[async_trait]
impl AsyncWrite for HttpUploader {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        let this = self.project();

        // Before writing, we poll the call future to see if it's ready and
        // allow it to progress.
        //
        // If it is ready, we should not be writing and we must fail.
        if let Poll::Ready(resp) = this.call.poll(cx) {
            return Poll::Ready(match resp {
                Ok(resp) => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    anyhow::anyhow!(
                        "HTTP request unexpectedly completed while uploading content: {}",
                        resp.status()
                    ),
                )),
                Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
            });
        }

        if let Some(w) = this.w.get_mut() {
            Pin::new(w).poll_write(cx, buf)
        } else {
            panic!("HttpUploader::poll_write called after completion")
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();

        // Before flushing, we poll the call future to see if it's ready and
        // allow it to progress.
        //
        // If it is ready, we should not be flushing and we must fail.
        if let Poll::Ready(resp) = this.call.poll(cx) {
            return Poll::Ready(match resp {
                Ok(resp) => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    anyhow::anyhow!(
                        "HTTP request unexpectedly completed while uploading content: {}",
                        resp.status()
                    ),
                )),
                Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
            });
        }

        if let Some(w) = this.w.get_mut() {
            Pin::new(w).poll_flush(cx)
        } else {
            panic!("HttpUploader::poll_flush called after completion")
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();
        let w = this.w.get_mut();

        match {
            if let Some(w) = w {
                Pin::new(w).poll_shutdown(cx)
            } else {
                Poll::Ready(Ok(()))
            }
        } {
            Poll::Ready(Ok(())) => {
                // Shutdown went fine: let's make sure we never poll the writer again.
                w.take();
            }
            Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
            Poll::Pending => return Poll::Pending,
        };

        match this.call.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(resp)) => {
                if resp.status().is_success() {
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        anyhow::anyhow!(
                            "HTTP request unexpectedly completed while uploading content: {}",
                            resp.status()
                        ),
                    )))
                }
            }
            Poll::Ready(Err(err)) => {
                Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, err)))
            }
        }
    }
}
