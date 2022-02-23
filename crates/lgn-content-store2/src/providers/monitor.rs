use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
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

pub trait TransferCallbacks<Id>: Debug + Send + Sync {
    fn on_transfer_avoided(&self, id: &Id, total: usize);
    fn on_transfer_started(&self, id: &Id, total: usize);
    fn on_transfer_progress(&self, id: &Id, total: usize, inc: usize, current: usize);
    fn on_transfer_stopped(
        &self,
        id: &Id,
        total: usize,
        inc: usize,
        current: usize,
        result: Result<()>,
    );
}

/// A `MonitorProvider` is a provider that tracks uploads and downloads.
#[derive(Debug)]
pub struct MonitorProvider<Inner> {
    inner: Inner,
    on_download_callbacks: Option<Arc<Box<dyn TransferCallbacks<Identifier>>>>,
    on_upload_callbacks: Option<Arc<Box<dyn TransferCallbacks<Identifier>>>>,
}

impl<Inner: Clone> Clone for MonitorProvider<Inner> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            on_download_callbacks: self.on_download_callbacks.clone(),
            on_upload_callbacks: self.on_upload_callbacks.clone(),
        }
    }
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

    pub fn on_download_callbacks(
        mut self,
        callbacks: impl TransferCallbacks<Identifier> + 'static,
    ) -> Self {
        self.on_download_callbacks = Some(Arc::new(Box::new(callbacks)));
        self
    }

    pub fn on_upload_callbacks(
        mut self,
        callbacks: impl TransferCallbacks<Identifier> + 'static,
    ) -> Self {
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
                id.data_size(),
                Arc::clone(callbacks),
            ))
        } else {
            reader
        })
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
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
                                id.data_size(),
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
                    callbacks.on_transfer_avoided(id, id.data_size());
                }

                Err(Error::AlreadyExists)
            }
            Err(err) => Err(err),
        }?;

        Ok(if let Some(callbacks) = &self.on_upload_callbacks {
            Box::pin(MonitorAsyncAdapter::new(
                writer,
                id.clone(),
                id.data_size(),
                Arc::clone(callbacks),
            ))
        } else {
            writer
        })
    }
}

#[pin_project]
pub struct MonitorAsyncAdapter<Inner, Id> {
    #[pin]
    inner: Inner,
    id: Id,
    #[pin]
    started: bool,
    total: usize,
    #[pin]
    current: usize,
    #[pin]
    inc: usize,
    progress_step: usize,
    callbacks: Arc<Box<dyn TransferCallbacks<Id>>>,
}

impl<Inner, Id> MonitorAsyncAdapter<Inner, Id> {
    pub fn new(
        inner: Inner,
        id: Id,
        total: usize,
        callbacks: Arc<Box<dyn TransferCallbacks<Id>>>,
    ) -> Self {
        Self {
            inner,
            id,
            started: false,
            total,
            current: 0,
            inc: 0,
            progress_step: total / 100,
            callbacks,
        }
    }
}

#[async_trait]
impl<Inner: AsyncRead + Send, Id> AsyncRead for MonitorAsyncAdapter<Inner, Id> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let mut this = self.project();

        if !*this.started {
            *this.started = true;
            this.callbacks.on_transfer_started(this.id, *this.total);
        }

        let before = buf.filled().len();

        match this.inner.poll_read(cx, buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(res) => {
                let diff = buf.filled().len() - before;

                if diff > 0 {
                    *this.inc += diff;

                    if *this.inc > *this.progress_step {
                        *this.current += *this.inc;

                        this.callbacks.on_transfer_progress(
                            this.id,
                            *this.total,
                            *this.inc,
                            *this.current,
                        );

                        *this.inc = 0;
                    }
                } else {
                    this.callbacks.on_transfer_stopped(
                        this.id,
                        *this.total,
                        *this.inc,
                        *this.current,
                        match &res {
                            Ok(_) => Ok(()),
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
impl<Inner: AsyncWrite + Send, Id> AsyncWrite for MonitorAsyncAdapter<Inner, Id> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let mut this = self.project();

        if !*this.started {
            *this.started = true;
            this.callbacks.on_transfer_started(this.id, *this.total);
        }

        match this.inner.poll_write(cx, buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(diff)) => {
                if diff > 0 {
                    *this.inc += diff;

                    if *this.inc > *this.progress_step {
                        *this.current += *this.inc;

                        this.callbacks.on_transfer_progress(
                            this.id,
                            *this.total,
                            *this.inc,
                            *this.current,
                        );

                        *this.inc = 0;
                    }
                }

                Poll::Ready(Ok(diff))
            }
            Poll::Ready(Err(err)) => {
                this.callbacks.on_transfer_stopped(
                    this.id,
                    *this.total,
                    *this.inc,
                    *this.current,
                    Err(anyhow::anyhow!("{}", err).into()),
                );

                Poll::Ready(Err(err))
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        let this = self.project();
        let res = this.inner.poll_flush(cx);

        if let Poll::Ready(Err(err)) = &res {
            this.callbacks.on_transfer_stopped(
                this.id,
                *this.total,
                *this.inc,
                *this.current,
                Err(anyhow::anyhow!("{}", err).into()),
            );
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
                this.callbacks.on_transfer_stopped(
                    this.id,
                    *this.total,
                    *this.inc,
                    *this.current,
                    Ok(()),
                );
            }
            Poll::Ready(Err(err)) => {
                this.callbacks.on_transfer_stopped(
                    this.id,
                    *this.total,
                    *this.inc,
                    *this.current,
                    Err(anyhow::anyhow!("{}", err).into()),
                );
            }
            Poll::Pending => {}
        }

        res
    }
}
