use async_trait::async_trait;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{BoxedAsyncRead, BoxedAsyncWrite};

use super::{BlobStorage, Result};

pub struct Lz4BlobStorageAdapter<B: BlobStorage> {
    inner: B,
    capacity: usize,
}

impl<B: BlobStorage> Lz4BlobStorageAdapter<B> {
    pub async fn new(inner: B, capacity: usize) -> Self {
        Self { inner, capacity }
    }
}

/// LZ4 decompression adapter.
///
/// This implementation is currently highly inneficient.
///
/// We are doing plenty of allocations and copying. Mostly because at the time
/// of writing, we have no hint as to what the final size of the decompressed
/// data will be and must allocate a new vector for each read. This would be
/// better solved by having a stream-oriented decompression API for LZ4 and
/// never allocating anything.
///
/// We don't care for know: we just want it to work.
#[pin_project]
struct DecompressionAsyncReader<R: AsyncRead> {
    #[pin]
    inner: R,
    bufs: VecList,
}

impl<R: AsyncRead> DecompressionAsyncReader<R> {
    fn new(inner: R) -> Self {
        let bufs = VecList::new();

        Self { inner, bufs }
    }
}

impl<R: AsyncRead> AsyncRead for DecompressionAsyncReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.project();
        let mut data = Vec::new();
        data.resize(buf.capacity(), 0);
        let mut tbuf = tokio::io::ReadBuf::new(data.as_mut_slice());

        match this.inner.poll_read(cx, &mut tbuf) {
            Poll::Ready(Ok(())) => {
                let size = tbuf.filled().len();
                drop(tbuf);

                // If we read 0 bytes, we're done and from then on, we must
                // write back all the data we stored.
                if size == 0 {
                    for (offset, b) in this.bufs.iter_mut() {
                        let bsize = b.len() - *offset;

                        if bsize > 0 {
                            if buf.remaining() >= bsize {
                                buf.put_slice(&b[*offset..]);
                                *offset = 0;
                                b.clear();
                            } else {
                                buf.put_slice(&b[*offset..*offset + buf.remaining()]);
                                *offset += buf.remaining();

                                break;
                            }
                        }
                    }
                } else {
                    data.truncate(size);
                    this.bufs.push(data);
                }

                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

struct ByteStreamWriter {
    client: aws_sdk_s3::Client,
    bucket_name: String,
    key: String,
    state: Mutex<ByteStreamWriterState>,
}

type ByteStreamWriterBoxedFuture = Box<
    dyn Future<
            Output = std::result::Result<
                aws_sdk_s3::output::PutObjectOutput,
                aws_sdk_s3::SdkError<aws_sdk_s3::error::PutObjectError>,
            >,
        > + Send
        + 'static,
>;

enum ByteStreamWriterState {
    Writing(Vec<u8>),
    Uploading(Pin<ByteStreamWriterBoxedFuture>),
}

impl ByteStreamWriter {
    fn new(client: aws_sdk_s3::Client, bucket_name: String, key: String) -> Self {
        Self {
            client,
            bucket_name,
            key,
            state: Mutex::new(ByteStreamWriterState::Writing(Vec::new())),
        }
    }

    fn poll_write_impl(&self, buf: &[u8]) -> Poll<std::result::Result<usize, std::io::Error>> {
        match &mut *self.state.lock().unwrap() {
            ByteStreamWriterState::Writing(buffer) => {
                buffer.extend_from_slice(buf);

                Poll::Ready(Ok(buf.len()))
            }
            ByteStreamWriterState::Uploading(_) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "cannot write to an uploading stream",
            ))),
        }
    }

    fn poll_flush_impl(&self) -> Poll<std::result::Result<(), std::io::Error>> {
        match &*self.state.lock().unwrap() {
            ByteStreamWriterState::Writing(_) => Poll::Ready(Ok(())),
            ByteStreamWriterState::Uploading(_) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "cannot flush an uploading stream",
            ))),
        }
    }

    fn poll_shutdown_impl(
        &self,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let mut state = self.state.lock().unwrap();

        let fut = match &mut *state {
            ByteStreamWriterState::Writing(buffer) => {
                let body = aws_sdk_s3::ByteStream::from(std::mem::take(buffer));

                let fut = self
                    .client
                    .put_object()
                    .bucket(&self.bucket_name)
                    .key(&self.key)
                    .body(body)
                    .send();

                *state = ByteStreamWriterState::Uploading(Box::pin(fut));

                if let ByteStreamWriterState::Uploading(fut) = &mut *state {
                    fut
                } else {
                    unreachable!()
                }
            }
            ByteStreamWriterState::Uploading(fut) => fut,
        };

        match fut.as_mut().poll(cx) {
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(())),
            Poll::Ready(Err(err)) => {
                Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, err)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for ByteStreamWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        self.poll_write_impl(buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        self.poll_flush_impl()
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        self.poll_shutdown_impl(cx)
    }
}

#[async_trait]
impl<B: BlobStorage> BlobStorage for Lz4BlobStorageAdapter<B> {
    async fn get_blob_reader(&self, hash: &str) -> Result<BoxedAsyncRead> {
        self.inner
            .get_blob_reader(hash)
            .await
            .map(|reader| Box::pin(DecompressionAsyncReader::new(reader)) as BoxedAsyncRead)
    }

    async fn get_blob_writer(&self, hash: &str) -> Result<Option<BoxedAsyncWrite>> {
        self.inner.get_blob_writer(hash).await
    }
}

/// A list of vectors of bytes that acts as an aggregator for sequential async
/// reads.
struct VecList {
    bufs: Vec<VecListItem>,
}

struct VecListItem {
    offset: usize,
    data: Vec<u8>,
}

impl VecList {
    fn new() -> Self {
        Self { bufs: Vec::new() }
    }

    fn push(&mut self, data: Vec<u8>) {
        if data.len() > 0 {
            self.bufs.push(VecListItem::new(data));
        }
    }

    fn move_to_buf(&mut self, buf: tokio::io::ReadBuf<'_>) {
        for item in self.bufs.iter_mut() {
            if !item.empty() {
                item.move_to_buf(buf);

                if buf.remaining() == 0 {
                    return;
                }
            }
        }
    }
}

impl VecListItem {
    fn new(data: Vec<u8>) -> Self {
        Self { offset: 0, data }
    }

    fn empty(&self) -> bool {
        self.size() == 0
    }

    fn size(&self) -> usize {
        self.data.len() - self.offset
    }

    fn data(&self) -> &[u8] {
        &self.data[self.offset..]
    }

    fn move_to_buf(&mut self, buf: tokio::io::ReadBuf<'_>) {
        if buf.remaining() >= self.size() {
            buf.put_slice(&self.data[self.offset..]);
            self.offset = 0;
            self.data.clear();
        } else {
            buf.put_slice(&self.data[self.offset..self.offset + buf.remaining()]);
            self.offset += buf.remaining();
        }
    }
}
