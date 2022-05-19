use std::{
    fmt::{Debug, Display},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use lgn_tracing::debug;
use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use super::{
    AsyncReadWithOriginAndSize, ContentAsyncReadWithOriginAndSize, ContentAsyncWrite,
    ContentReader, ContentWriter, Error, HashRef, Origin, Result,
};

pub trait TransferCallbacks<Id = HashRef>: Debug + Send + Sync {
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

// A blanket implementation of TransferCallbacks for Arc<T>.
impl<Id, T: TransferCallbacks<Id> + ?Sized> TransferCallbacks<Id> for Arc<T> {
    fn on_transfer_avoided(&self, id: &Id, total: usize) {
        (**self).on_transfer_avoided(id, total);
    }

    fn on_transfer_started(&self, id: &Id, total: usize) {
        (**self).on_transfer_started(id, total);
    }

    fn on_transfer_progress(&self, id: &Id, total: usize, inc: usize, current: usize) {
        (**self).on_transfer_progress(id, total, inc, current);
    }

    fn on_transfer_stopped(
        &self,
        id: &Id,
        total: usize,
        inc: usize,
        current: usize,
        result: Result<()>,
    ) {
        (**self).on_transfer_stopped(id, total, inc, current, result);
    }
}

// A blanket implementation of TransferCallbacks for &T.
impl<Id, T: TransferCallbacks<Id> + ?Sized> TransferCallbacks<Id> for &T {
    fn on_transfer_avoided(&self, id: &Id, total: usize) {
        (**self).on_transfer_avoided(id, total);
    }

    fn on_transfer_started(&self, id: &Id, total: usize) {
        (**self).on_transfer_started(id, total);
    }

    fn on_transfer_progress(&self, id: &Id, total: usize, inc: usize, current: usize) {
        (**self).on_transfer_progress(id, total, inc, current);
    }

    fn on_transfer_stopped(
        &self,
        id: &Id,
        total: usize,
        inc: usize,
        current: usize,
        result: Result<()>,
    ) {
        (**self).on_transfer_stopped(id, total, inc, current, result);
    }
}

/// A `ContentProviderMonitor` is a provider that tracks uploads and downloads.
#[derive(Debug)]
pub struct ContentProviderMonitor<Inner> {
    inner: Inner,
    on_download_callbacks: Option<Arc<Box<dyn TransferCallbacks>>>,
    on_upload_callbacks: Option<Arc<Box<dyn TransferCallbacks>>>,
}

impl<Inner: Display> Display for ContentProviderMonitor<Inner> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "monitoring for {}", self.inner)
    }
}

impl<Inner: Clone> Clone for ContentProviderMonitor<Inner> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            on_download_callbacks: self.on_download_callbacks.clone(),
            on_upload_callbacks: self.on_upload_callbacks.clone(),
        }
    }
}

impl<Inner> ContentProviderMonitor<Inner> {
    /// Creates a new `MemoryProvider` instance who stores content in the
    /// process memory.
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            on_download_callbacks: None,
            on_upload_callbacks: None,
        }
    }

    /// Clear the download callbacks.
    pub fn clear_download_callbacks(&mut self) {
        self.on_download_callbacks = None;
    }

    /// Set the download callbacks.
    pub fn set_download_callbacks(&mut self, callbacks: impl TransferCallbacks<HashRef> + 'static) {
        self.on_download_callbacks = Some(Arc::new(Box::new(callbacks)));
    }

    /// Clear the upload callbacks.
    pub fn clear_upload_callbacks(&mut self) {
        self.on_upload_callbacks = None;
    }

    /// Set the upload callbacks.
    pub fn set_upload_callbacks(&mut self, callbacks: impl TransferCallbacks<HashRef> + 'static) {
        self.on_upload_callbacks = Some(Arc::new(Box::new(callbacks)));
    }
}

#[async_trait]
impl<Inner: ContentReader + Send + Sync> ContentReader for ContentProviderMonitor<Inner> {
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        let reader = self.inner.get_content_reader(id).await?;

        Ok(if let Some(callbacks) = &self.on_download_callbacks {
            let m =
                MonitorAsyncAdapter::new(reader, id.clone(), id.data_size(), Arc::clone(callbacks));

            Box::pin(m)
        } else {
            reader
        })
    }
}

#[async_trait]
impl<Inner: ContentWriter + Send + Sync> ContentWriter for ContentProviderMonitor<Inner> {
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        let writer = match self.inner.get_content_writer(id).await {
            Ok(writer) => Ok(writer),
            Err(Error::HashRefAlreadyExists(id)) => {
                if let Some(callbacks) = &self.on_upload_callbacks {
                    callbacks.on_transfer_avoided(&id, id.data_size());
                }

                Err(Error::HashRefAlreadyExists(id))
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
    #[pin]
    stopped: bool,
    total: usize,
    #[pin]
    current: usize,
    #[pin]
    inc: usize,
    progress_step: usize,
    callbacks: Arc<Box<dyn TransferCallbacks<Id>>>,
}

impl<Inner, Id: Display> MonitorAsyncAdapter<Inner, Id> {
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
            stopped: false,
            total,
            current: 0,
            inc: 0,
            progress_step: total / 100,
            callbacks,
        }
    }
}

impl<Id: Display> AsyncReadWithOriginAndSize
    for MonitorAsyncAdapter<ContentAsyncReadWithOriginAndSize, Id>
{
    fn size(&self) -> usize {
        self.inner.size()
    }

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

        let res = this.inner.poll_read(cx, buf);

        if !*this.stopped {
            // Monitoring only: we do not actually use or move `res` and make sure
            // it cannot happen by overriding it in the scope.
            if let Poll::Ready(res) = &res {
                let diff = buf.filled().len() - before;

                if diff > 0 {
                    *this.inc += diff;
                    *this.current += diff;

                    if *this.inc > *this.progress_step {
                        this.callbacks.on_transfer_progress(
                            this.id,
                            *this.total,
                            *this.inc,
                            *this.current,
                        );

                        *this.inc = 0;
                    }
                } else {
                    // A null difference means that we reached the end of the
                    // stream.
                    *this.stopped = true;

                    match res {
                        Ok(_) => {
                            if *this.current == *this.total {
                                debug!(
                                    "MonitorAsyncAdapter::poll_read: transfer completed for {}",
                                    this.id
                                );

                                this.callbacks.on_transfer_stopped(
                                    this.id,
                                    *this.total,
                                    *this.inc,
                                    *this.current,
                                    Ok(()),
                                );
                            } else {
                                debug!(
                                    "MonitorAsyncAdapter::poll_read: transfer reported EOF ({}/{}) for {}",
                                    *this.current,
                                    *this.total,
                                    this.id,
                                );

                                this.callbacks.on_transfer_stopped(
                                    this.id,
                                    *this.total,
                                    *this.inc,
                                    *this.current,
                                    Err(std::io::Error::new(
                                        std::io::ErrorKind::UnexpectedEof,
                                        "Unexpected EOF",
                                    )
                                    .into()),
                                );
                            }
                        }
                        Err(err) => {
                            debug!(
                                "MonitorAsyncAdapter::poll_read: transfer failed for {}: {}",
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
            }
        }

        res
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
