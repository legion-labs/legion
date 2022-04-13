use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Debug, Display},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use lgn_tracing::debug;
use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::{
    traits::AsyncReadWithOrigin, ContentAsyncReadWithOrigin, ContentAsyncWrite, ContentReader,
    ContentTracker, ContentWriter, Error, Identifier, Origin, Result,
};

pub trait TransferCallbacks<Id = Identifier>: Debug + Send + Sync {
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

pub trait AliasCallbacks: Debug + Send + Sync {
    fn on_alias_registered(&self, key_space: &str, key: &str, id: &Identifier);
}

pub trait TrackingCallbacks<Id = Identifier>: Debug + Send + Sync {
    fn on_reference_count_increased(&self, id: &Id);
    fn on_reference_count_decreased(&self, id: &Id);
    fn on_references_popped(&self, id: &[Id]);
}

/// A `MonitorProvider` is a provider that tracks uploads and downloads.
#[derive(Debug)]
pub struct MonitorProvider<Inner> {
    inner: Inner,
    on_download_callbacks: Option<Arc<Box<dyn TransferCallbacks>>>,
    on_upload_callbacks: Option<Arc<Box<dyn TransferCallbacks>>>,
    on_alias_callbacks: Option<Arc<Box<dyn AliasCallbacks>>>,
    on_tracking_callbacks: Option<Arc<Box<dyn TrackingCallbacks>>>,
}

impl<Inner: Display> Display for MonitorProvider<Inner> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "monitoring for {}", self.inner)
    }
}

impl<Inner: Clone> Clone for MonitorProvider<Inner> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            on_download_callbacks: self.on_download_callbacks.clone(),
            on_upload_callbacks: self.on_upload_callbacks.clone(),
            on_alias_callbacks: self.on_alias_callbacks.clone(),
            on_tracking_callbacks: self.on_tracking_callbacks.clone(),
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
            on_alias_callbacks: None,
            on_tracking_callbacks: None,
        }
    }

    #[must_use]
    pub fn on_download_callbacks(
        mut self,
        callbacks: impl TransferCallbacks<Identifier> + 'static,
    ) -> Self {
        self.on_download_callbacks = Some(Arc::new(Box::new(callbacks)));
        self
    }

    #[must_use]
    pub fn on_upload_callbacks(
        mut self,
        callbacks: impl TransferCallbacks<Identifier> + 'static,
    ) -> Self {
        self.on_upload_callbacks = Some(Arc::new(Box::new(callbacks)));
        self
    }
}

impl<Inner: ContentWriter> MonitorProvider<Inner> {
    #[must_use]
    pub fn on_alias_callbacks(mut self, callbacks: impl AliasCallbacks + 'static) -> Self {
        self.on_alias_callbacks = Some(Arc::new(Box::new(callbacks)));
        self
    }
}

impl<Inner: ContentTracker> MonitorProvider<Inner> {
    #[must_use]
    pub fn on_tracking_callbacks(mut self, callbacks: impl TrackingCallbacks + 'static) -> Self {
        self.on_tracking_callbacks = Some(Arc::new(Box::new(callbacks)));
        self
    }
}

#[async_trait]
impl<Inner: ContentReader + Send + Sync> ContentReader for MonitorProvider<Inner> {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin> {
        let reader = self.inner.get_content_reader(id).await?;

        Ok(if let Some(callbacks) = &self.on_download_callbacks {
            let m = MonitorAsyncAdapter::new(
                reader,
                id.clone(),
                id.data_size(),
                Arc::clone(callbacks),
                None,
            );

            Box::pin(m)
        } else {
            reader
        })
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
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
                                None,
                            ))
                                as ContentAsyncReadWithOrigin),
                            Err(err) => Err(err),
                        },
                    )
                })
                .collect()
        } else {
            readers
        })
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        self.inner.resolve_alias(key_space, key).await
    }
}

#[async_trait]
impl<Inner: ContentWriter + Send + Sync> ContentWriter for MonitorProvider<Inner> {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        let writer = match self.inner.get_content_writer(id).await {
            Ok(writer) => Ok(writer),
            Err(Error::IdentifierAlreadyExists(_)) => {
                if let Some(callbacks) = &self.on_upload_callbacks {
                    callbacks.on_transfer_avoided(id, id.data_size());
                }

                Err(Error::IdentifierAlreadyExists(id.clone()))
            }
            Err(err) => Err(err),
        }?;

        Ok(if let Some(callbacks) = &self.on_upload_callbacks {
            Box::pin(MonitorAsyncAdapter::new(
                writer,
                id.clone(),
                id.data_size(),
                Arc::clone(callbacks),
                None,
            ))
        } else {
            writer
        })
    }

    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        self.inner.register_alias(key_space, key, id).await?;

        if let Some(callbacks) = &self.on_alias_callbacks {
            callbacks.on_alias_registered(key_space, key, id);
        }

        Ok(())
    }
}

#[async_trait]
impl<Inner: ContentTracker + Send + Sync> ContentTracker for MonitorProvider<Inner> {
    async fn remove_content(&self, id: &Identifier) -> Result<()> {
        self.inner.remove_content(id).await?;

        if let Some(callbacks) = &self.on_tracking_callbacks {
            callbacks.on_reference_count_decreased(id);
        }

        Ok(())
    }

    async fn pop_referenced_identifiers(&self) -> Result<Vec<Identifier>> {
        let ids = self.inner.pop_referenced_identifiers().await?;

        if let Some(callbacks) = &self.on_tracking_callbacks {
            callbacks.on_references_popped(&ids);
        }

        Ok(ids)
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
    tracking_callbacks: Option<Arc<Box<dyn TrackingCallbacks<Id>>>>,
}

impl<Inner, Id: Display> MonitorAsyncAdapter<Inner, Id> {
    pub fn new(
        inner: Inner,
        id: Id,
        total: usize,
        callbacks: Arc<Box<dyn TransferCallbacks<Id>>>,
        tracking_callbacks: Option<Arc<Box<dyn TrackingCallbacks<Id>>>>,
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
            tracking_callbacks,
        }
    }
}

impl<Id: Display> AsyncReadWithOrigin for MonitorAsyncAdapter<ContentAsyncReadWithOrigin, Id> {
    fn origin(&self) -> &Origin {
        self.inner.origin()
    }
}

#[async_trait]
impl<Inner: AsyncRead + Send, Id: Display> AsyncRead for MonitorAsyncAdapter<Inner, Id> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let mut this = self.project();

        if !*this.started {
            *this.started = true;

            debug!(
                "MonitorAsyncAdapter::poll_read: transfer started for {}",
                this.id
            );

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
                    match &res {
                        Ok(_) => {
                            debug!(
                                "MonitorAsyncAdapter::poll_read: transfer stopped for {}",
                                this.id
                            );

                            this.callbacks.on_transfer_stopped(
                                this.id,
                                *this.total,
                                *this.inc,
                                *this.current,
                                Ok(()),
                            );
                        }
                        Err(err) => {
                            debug!(
                                "MonitorAsyncAdapter::poll_read: transfer stopped for {}: {}",
                                this.id, err
                            );

                            this.callbacks.on_transfer_stopped(
                                this.id,
                                *this.total,
                                *this.inc,
                                *this.current,
                                Err(anyhow::anyhow!("{}", err).into()),
                            );
                        }
                    }
                }

                Poll::Ready(res)
            }
        }
    }
}

#[async_trait]
impl<Inner: AsyncWrite + Send, Id: Display> AsyncWrite for MonitorAsyncAdapter<Inner, Id> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let mut this = self.project();

        if !*this.started {
            *this.started = true;

            debug!(
                "MonitorAsyncAdapter::poll_write: transfer started for {}",
                this.id
            );

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
                debug!(
                    "MonitorAsyncAdapter::poll_write: transfer stopped for {}: {}",
                    this.id, err
                );

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
            debug!(
                "MonitorAsyncAdapter::poll_flush: transfer stopped for {}: {}",
                this.id, err
            );

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
                debug!(
                    "MonitorAsyncAdapter::poll_shutdown: transfer stopped for {}",
                    this.id
                );

                if let Some(callbacks) = this.tracking_callbacks {
                    callbacks.on_reference_count_increased(this.id);
                }

                this.callbacks.on_transfer_stopped(
                    this.id,
                    *this.total,
                    *this.inc,
                    *this.current,
                    Ok(()),
                );
            }
            Poll::Ready(Err(err)) => {
                debug!(
                    "MonitorAsyncAdapter::poll_shutdown: transfer stopped for {}: {}",
                    this.id, err
                );

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
