use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures::Future;
use pin_project::pin_project;
use tokio::io::AsyncWrite;

use crate::{Identifier, Result};

#[pin_project]
pub struct Uploader<Impl: UploaderImpl> {
    #[pin]
    state: State<Impl>,
}

#[allow(clippy::type_complexity)]
enum State<Impl> {
    Writing(Option<(std::io::Cursor<Vec<u8>>, Identifier, Impl)>),
    Uploading(Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'static>>),
}

#[async_trait]
pub trait UploaderImpl: Unpin + Send + Sync + 'static {
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()>;
}

impl<Impl: UploaderImpl> Uploader<Impl> {
    pub fn new(id: Identifier, impl_: Impl) -> Self {
        let state = State::Writing(Some((std::io::Cursor::new(Vec::new()), id, impl_)));

        Self { state }
    }

    async fn upload(data: Vec<u8>, id: Identifier, impl_: Impl) -> Result<(), std::io::Error> {
        id.matches(&data).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                anyhow::anyhow!("the data does not match the specified id: {}", err),
            )
        })?;

        impl_
            .upload(data, id)
            .await
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
    }
}

impl<Impl: UploaderImpl> AsyncWrite for Uploader<Impl> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        let this = self.project();

        if let State::Writing(Some((cursor, _, _))) = this.state.get_mut() {
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

        if let State::Writing(Some((cursor, _, _))) = this.state.get_mut() {
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
            *state = match state {
                State::Writing(args) => {
                    let res = Pin::new(&mut args.as_mut().unwrap().0).poll_shutdown(cx);

                    match res {
                        Poll::Ready(Ok(())) => {
                            let (cursor, id, impl_) = args.take().unwrap();

                            State::Uploading(Box::pin(Self::upload(cursor.into_inner(), id, impl_)))
                        }
                        p => return p,
                    }
                }
                State::Uploading(call) => return Pin::new(call).poll(cx),
            };
        }
    }
}
