use std::{
    cmp::min,
    collections::{BTreeMap, BTreeSet},
    pin::Pin,
    task::{Context, Poll},
};

use async_trait::async_trait;
use lgn_tracing::warn;
use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::{
    AliasProvider, AliasRegisterer, AliasResolver, ContentAsyncRead, ContentAsyncWrite,
    ContentProvider, ContentReader, ContentWriter, Error, Identifier, Result,
};

/// A `LocalProvider` is a provider that stores content on the local filesystem.
#[derive(Debug, Clone)]
pub struct CachingProvider<Remote, Local> {
    remote: Remote,
    local: Local,
}

impl<Remote, Local> CachingProvider<Remote, Local> {
    /// Creates a new `CachingProvider` instance who stores content in the
    /// backing remote and local providers.
    pub fn new(remote: Remote, local: Local) -> Self {
        Self { remote, local }
    }
}

#[async_trait]
impl<Remote: AliasResolver + Send + Sync, Local: AliasProvider + Send + Sync> AliasResolver
    for CachingProvider<Remote, Local>
{
    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        match self.local.resolve_alias(key_space, key).await {
            Ok(id) => Ok(id),
            Err(Error::NotFound) => match self.remote.resolve_alias(key_space, key).await {
                Ok(id) => {
                    if let Err(err) = self.local.register_alias(key_space, key, &id).await {
                        warn!(
                            "Failed to register alias {}/{} in local cache: {}",
                            key_space, key, err
                        );
                    }

                    Ok(id)
                }
                Err(err) => Err(err),
            },
            // If the local provider fails, we just fall back to the remote without caching.
            Err(_) => self.remote.resolve_alias(key_space, key).await,
        }
    }
}

#[async_trait]
impl<Remote: AliasRegisterer + Send + Sync, Local: AliasRegisterer + Send + Sync> AliasRegisterer
    for CachingProvider<Remote, Local>
{
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        self.remote.register_alias(key_space, key, id).await?;

        if let Err(err) = self.local.register_alias(key_space, key, id).await {
            warn!(
                "Failed to register alias {}/{} in local cache: {}",
                key_space, key, err
            );
        }

        Ok(())
    }
}

#[async_trait]
impl<Remote: ContentReader + Send + Sync, Local: ContentProvider + Send + Sync> ContentReader
    for CachingProvider<Remote, Local>
{
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        match self.local.get_content_reader(id).await {
            Ok(reader) => Ok(reader),
            Err(Error::NotFound) => {
                match self.remote.get_content_reader(id).await {
                    Ok(reader) => {
                        let writer = match self.local.get_content_writer(id).await {
                            Ok(writer) => writer,
                            Err(_) => {
                                // If we fail to get a writer, we should just return the reader.
                                //
                                // This covers the race condition where the
                                // local provider got the asset since we first
                                // tried to read it.
                                return Ok(reader);
                            }
                        };

                        Ok(Box::pin(TeeAsyncRead::new(reader, writer, id.data_size()))
                            as ContentAsyncRead)
                    }
                    Err(err) => Err(err),
                }
            }
            // If the local provider fails, we just fall back to the remote without caching.
            Err(_) => self.remote.get_content_reader(id).await,
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        // If we can't make the request at all, try on the remote one without caching.
        let mut readers = match self.local.get_content_readers(ids).await {
            Ok(readers) => readers,
            Err(_) => return self.remote.get_content_readers(ids).await,
        };

        let missing_ids = readers
            .iter()
            .filter_map(|(id, reader)| {
                if let Err(Error::NotFound) = reader {
                    Some(id)
                } else {
                    None
                }
            })
            .copied()
            .cloned()
            .collect::<BTreeSet<_>>();

        let mut missing_writers = BTreeMap::new();

        for missing_id in &missing_ids {
            if let Ok(writer) = self.local.get_content_writer(missing_id).await {
                missing_writers.insert(missing_id, writer);
            }
        }

        if !missing_ids.is_empty() {
            readers.extend(
                self.remote
                    .get_content_readers(&missing_ids)
                    .await?
                    .into_iter()
                    .map(|(i, reader)| match reader {
                        Ok(reader) => {
                            if let Some(writer) = missing_writers.remove(i) {
                                (
                                    ids.get(i).unwrap(),
                                    Ok(Box::pin(TeeAsyncRead::new(reader, writer, i.data_size()))
                                        as ContentAsyncRead),
                                )
                            } else {
                                (ids.get(i).unwrap(), Ok(reader))
                            }
                        }
                        Err(err) => (ids.get(i).unwrap(), Err(err)),
                    }),
            );
        }

        Ok(readers)
    }
}

#[async_trait]
impl<Remote: ContentWriter + Send + Sync, Local: ContentWriter + Send + Sync> ContentWriter
    for CachingProvider<Remote, Local>
{
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        let remote_writer = self.remote.get_content_writer(id).await?;

        match self.local.get_content_writer(id).await {
            Ok(writer) => Ok(
                Box::pin(MultiAsyncWrite::new(remote_writer, writer, id.data_size()))
                    as ContentAsyncWrite,
            ),
            Err(_) => Ok(remote_writer),
        }
    }
}

#[pin_project]
struct TeeAsyncRead<R, W> {
    #[pin]
    reader: R,
    #[pin]
    writer: W,
    #[pin]
    read_buffer: Vec<u8>,
    #[pin]
    read_cursor: usize,
    #[pin]
    copy_cursor: usize,
    #[pin]
    write_cursor: usize,
    #[pin]
    write_complete: bool,
    size: usize,
}

impl<R, W> TeeAsyncRead<R, W> {
    fn new(reader: R, writer: W, size: usize) -> Self {
        Self {
            reader,
            writer,
            read_buffer: vec![0; size],
            read_cursor: 0,
            copy_cursor: 0,
            write_cursor: 0,
            write_complete: false,
            size,
        }
    }
}

impl<R: AsyncRead + Send, W: AsyncWrite + Send> AsyncRead for TeeAsyncRead<R, W> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let mut this = self.project();

        if *this.read_cursor < *this.size {
            // We still need to read.

            let mut rbuf = ReadBuf::new(this.read_buffer.as_mut_slice());
            rbuf.set_filled(*this.read_cursor);

            match this.reader.poll_read(cx, &mut rbuf) {
                Poll::Ready(Ok(())) => {
                    *this.read_cursor = rbuf.filled().len();
                }
                Poll::Ready(Err(err)) => {
                    // Errors are fatal: we return it and it's over.
                    return Poll::Ready(Err(err));
                }
                Poll::Pending => {
                    // Nothing to read? That's fine, we'll attempt a write in the meanwhile.
                }
            }
        }

        if !*this.write_complete {
            // We always try to write what we can.
            if *this.write_cursor < *this.read_cursor {
                match this
                    .writer
                    .poll_write(cx, &this.read_buffer[*this.write_cursor..*this.read_cursor])
                {
                    Poll::Ready(Ok(num_bytes_written)) => {
                        *this.write_cursor += num_bytes_written;
                    }
                    Poll::Ready(Err(err)) => {
                        // Errors are fatal: we return it and it's over.
                        return Poll::Ready(Err(err));
                    }
                    Poll::Pending => {
                        // Not ready yet? That's fine, we'll try later.
                    }
                }
            } else if *this.write_cursor >= *this.size {
                // We have written everything we can, and we're done.
                match this.writer.poll_shutdown(cx) {
                    Poll::Ready(Ok(())) => {
                        *this.write_complete = true;
                    }
                    Poll::Ready(Err(err)) => {
                        // Errors are fatal: we return it and it's over.
                        return Poll::Ready(Err(err));
                    }
                    Poll::Pending => {
                        // Not ready yet? That's fine, we'll try later.
                    }
                }
            }
        }

        if *this.copy_cursor < *this.read_cursor {
            // Pass on some of what we've read so far.
            let num_bytes_to_copy = min(*this.read_cursor - *this.copy_cursor, buf.remaining());

            // We never want to return Ok() if we are not really writing
            // anything or it will incorrectly signal EOF to the caller.
            if num_bytes_to_copy > 0 {
                buf.put_slice(
                    &this.read_buffer[*this.copy_cursor..*this.copy_cursor + num_bytes_to_copy],
                );

                *this.copy_cursor += num_bytes_to_copy;

                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        } else if *this.write_complete {
            // If the write is complete, it means the read is complete too and
            // since in this branch the copy cursor is equal to the read cursor,
            // we can return EOF.
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}

#[pin_project]
struct MultiAsyncWrite<W1, W2> {
    #[pin]
    writer1: W1,
    #[pin]
    writer2: W2,
    #[pin]
    write1_cursor: usize,
    #[pin]
    write1_complete: bool,
    #[pin]
    write2_cursor: usize,
    #[pin]
    write2_complete: bool,
    size: usize,
}

impl<W1: AsyncWrite + Send, W2: AsyncWrite + Send> MultiAsyncWrite<W1, W2> {
    fn new(writer1: W1, writer2: W2, size: usize) -> Self {
        Self {
            writer1,
            writer2,
            write1_cursor: 0,
            write1_complete: false,
            write2_cursor: 0,
            write2_complete: false,
            size,
        }
    }
}

impl<W1: AsyncWrite + Send, W2: AsyncWrite + Send> AsyncWrite for MultiAsyncWrite<W1, W2> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        let mut this = self.project();
        let cursor = min(*this.write1_cursor, *this.write2_cursor);

        if *this.write1_cursor - cursor < buf.len() {
            match this
                .writer1
                .poll_write(cx, &buf[*this.write1_cursor - cursor..])
            {
                Poll::Ready(Ok(num_bytes_written)) => {
                    *this.write1_cursor += num_bytes_written;
                }
                Poll::Ready(Err(err)) => {
                    // Errors are fatal: we return it and it's over.
                    return Poll::Ready(Err(err));
                }
                Poll::Pending => {
                    // Not ready yet? That's fine, we'll try later.
                }
            }
        }

        if *this.write2_cursor - cursor < buf.len() {
            match this
                .writer2
                .poll_write(cx, &buf[*this.write2_cursor - cursor..])
            {
                Poll::Ready(Ok(num_bytes_written)) => {
                    *this.write2_cursor += num_bytes_written;
                }
                Poll::Ready(Err(err)) => {
                    // Errors are fatal: we return it and it's over.
                    return Poll::Ready(Err(err));
                }
                Poll::Pending => {
                    // Not ready yet? That's fine, we'll try later.
                }
            }
        }

        let num_bytes_written = min(*this.write1_cursor, *this.write2_cursor) - cursor;

        if num_bytes_written > 0 {
            Poll::Ready(Ok(num_bytes_written))
        } else if *this.write1_complete && *this.write2_complete {
            Poll::Ready(Ok(0))
        } else {
            Poll::Pending
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();

        let res1 = if !*this.write1_complete {
            this.writer1.poll_flush(cx)
        } else {
            Poll::Ready(Ok(()))
        };
        let res2 = if !*this.write2_complete {
            this.writer2.poll_flush(cx)
        } else {
            Poll::Ready(Ok(()))
        };

        match (res1, res2) {
            (Poll::Ready(Ok(())), Poll::Ready(Ok(()))) => Poll::Ready(Ok(())),
            (Poll::Ready(Err(err)), _) | (_, Poll::Ready(Err(err))) => Poll::Ready(Err(err)),
            _ => Poll::Pending,
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let mut this = self.project();

        let res1 = if !*this.write1_complete {
            let res = this.writer1.poll_shutdown(cx);

            if let Poll::Ready(Ok(())) = res {
                *this.write1_complete = true;
            }

            res
        } else {
            Poll::Ready(Ok(()))
        };
        let res2 = if !*this.write2_complete {
            let res = this.writer2.poll_shutdown(cx);

            if let Poll::Ready(Ok(())) = res {
                *this.write2_complete = true;
            }

            res
        } else {
            Poll::Ready(Ok(()))
        };

        match (res1, res2) {
            (Poll::Ready(Ok(())), Poll::Ready(Ok(()))) => Poll::Ready(Ok(())),
            (Poll::Ready(Err(err)), _) | (_, Poll::Ready(Err(err))) => Poll::Ready(Err(err)),
            _ => Poll::Pending,
        }
    }
}
