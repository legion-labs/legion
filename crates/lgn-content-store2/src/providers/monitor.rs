use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::{
    ContentAsyncRead, ContentAsyncWrite, ContentReader, ContentWriter, Error, Identifier, Result,
};

pub trait TransferCallbacks: Send + Sync {
    fn on_transfer_avoided(&self, id: &Identifier);
    fn on_transfer_started(&self, id: &Identifier);
    fn on_transfer_progress(&self, id: &Identifier, increment: usize, total: usize);
    fn on_transfer_stopped(&self, id: &Identifier, result: Result<usize>);
}

/// A `MonitorProvider` is a provider that tracks uploads and downloads.
pub struct MonitorProvider<Inner> {
    inner: Inner,
    on_download_callbacks: Option<Arc<Box<dyn TransferCallbacks>>>,
    on_upload_callbacks: Option<Arc<Box<dyn TransferCallbacks>>>,
}

impl<Inner> MonitorProvider<Inner> {
    /// Creates a new `MemoryProvider` instance who stores content in the
    /// process memory.
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            on_download_callbacks: None,
            on_upload_callbacks: None,
        }
    }

    pub fn on_download_callbacks(mut self, callbacks: impl TransferCallbacks + 'static) -> Self {
        self.on_download_callbacks = Some(Arc::new(Box::new(callbacks)));
        self
    }

    pub fn on_upload_callbacks(mut self, callbacks: impl TransferCallbacks + 'static) -> Self {
        self.on_upload_callbacks = Some(Arc::new(Box::new(callbacks)));
        self
    }
}

#[async_trait]
impl<Inner: ContentReader + Send + Sync> ContentReader for MonitorProvider<Inner> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        let reader = self.inner.get_content_reader(id).await?;

        Ok(if let Some(callbacks) = &self.on_download_callbacks {
            Box::pin(MonitorAsyncAdapter::new(
                reader,
                id.clone(),
                Arc::clone(callbacks),
            ))
        } else {
            reader
        })
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids [Identifier],
    ) -> Result<Vec<(&'ids Identifier, Result<ContentAsyncRead>)>> {
        let readers = self.inner.get_content_readers(ids).await?;

        Ok(if let Some(callbacks) = &self.on_download_callbacks {
            readers
                .into_iter()
                .map(|(id, reader)| {
                    (
                        id,
                        match reader {
                            Ok(reader) => Ok(Box::pin(MonitorAsyncAdapter::new(
                                reader,
                                id.clone(),
                                Arc::clone(callbacks),
                            )) as ContentAsyncRead),
                            Err(err) => Err(err),
                        },
                    )
                })
                .collect()
        } else {
            readers
        })
    }
}

#[async_trait]
impl<Inner: ContentWriter + Send + Sync> ContentWriter for MonitorProvider<Inner> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        let writer = match self.inner.get_content_writer(id).await {
            Ok(writer) => Ok(writer),
            Err(Error::AlreadyExists) => {
                if let Some(callbacks) = &self.on_upload_callbacks {
                    callbacks.on_transfer_avoided(id);
                }

                Err(Error::AlreadyExists)
            }
            Err(err) => Err(err),
        }?;

        Ok(if let Some(callbacks) = &self.on_upload_callbacks {
            Box::pin(MonitorAsyncAdapter::new(
                writer,
                id.clone(),
                Arc::clone(callbacks),
            ))
        } else {
            writer
        })
    }
}

#[pin_project]
struct MonitorAsyncAdapter<Inner> {
    #[pin]
    inner: Inner,
    id: Identifier,
    #[pin]
    total: usize,
    callbacks: Arc<Box<dyn TransferCallbacks>>,
}

impl<Inner> MonitorAsyncAdapter<Inner> {
    fn new(inner: Inner, id: Identifier, callbacks: Arc<Box<dyn TransferCallbacks>>) -> Self {
        callbacks.on_transfer_started(&id);

        Self {
            inner,
            id,
            total: 0,
            callbacks,
        }
    }
}

#[async_trait]
impl<Inner: AsyncRead + Send> AsyncRead for MonitorAsyncAdapter<Inner> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let mut this = self.project();

        match this.inner.poll_read(cx, buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(res) => {
                let inc = buf.filled().len();

                if inc > 0 {
                    *this.total += inc;
                    this.callbacks
                        .on_transfer_progress(this.id, inc, *this.total);
                } else {
                    this.callbacks.on_transfer_stopped(
                        this.id,
                        match &res {
                            Ok(_) => Ok(inc),
                            Err(err) => Err(anyhow::anyhow!("{}", err).into()),
                        },
                    );
                }

                Poll::Ready(res)
            }
        }
    }
}

#[async_trait]
impl<Inner: AsyncWrite + Send> AsyncWrite for MonitorAsyncAdapter<Inner> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let mut this = self.project();

        match this.inner.poll_write(cx, buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(inc)) => {
                if inc > 0 {
                    *this.total += inc;
                    this.callbacks
                        .on_transfer_progress(this.id, inc, *this.total);
                }

                Poll::Ready(Ok(inc))
            }
            Poll::Ready(Err(err)) => {
                this.callbacks
                    .on_transfer_stopped(this.id, Err(anyhow::anyhow!("{}", err).into()));

                Poll::Ready(Err(err))
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        let this = self.project();
        let res = this.inner.poll_flush(cx);

        if let Poll::Ready(Err(err)) = &res {
            this.callbacks
                .on_transfer_stopped(this.id, Err(anyhow::anyhow!("{}", err).into()));
        }

        res
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let this = self.project();
        let res = this.inner.poll_shutdown(cx);

        match &res {
            Poll::Ready(Ok(_)) => {
                this.callbacks.on_transfer_stopped(this.id, Ok(*this.total));
            }
            Poll::Ready(Err(err)) => {
                this.callbacks
                    .on_transfer_stopped(this.id, Err(anyhow::anyhow!("{}", err).into()));
            }
            Poll::Pending => {}
        }

        res
    }
}
