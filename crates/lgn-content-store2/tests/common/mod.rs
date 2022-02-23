mod asserts;
mod providers;

use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) use asserts::*;
use futures::Stream;
use hyper::server::{
    accept::Accept,
    conn::{AddrIncoming, AddrStream},
};
pub(crate) use providers::*;

pub struct TcpIncoming {
    inner: AddrIncoming,
}

impl TcpIncoming {
    pub(crate) fn new() -> Result<Self, anyhow::Error> {
        let mut inner = AddrIncoming::bind(&"127.0.0.1:0".parse()?)?;
        inner.set_nodelay(true);
        Ok(Self { inner })
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        self.inner.local_addr()
    }
}

impl Stream for TcpIncoming {
    type Item = Result<AddrStream, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_accept(cx)
    }
}
