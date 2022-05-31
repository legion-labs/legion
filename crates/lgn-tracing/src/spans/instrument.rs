//! Async instrumentation

#![allow(clippy::use_self)]

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::dispatch::{on_begin_scope, on_end_scope};

use super::SpanMetadata;

pin_project! {
    pub struct Instrumentation<T> {
        #[pin]
        inner: T,
        span: &'static SpanMetadata,
        // An [`Instrumentation`] is idle when it has been polled at least once
        // and that the inner Future returned `Poll::Pending`.
        // As soon as the inner Future returns `Poll::Ready` this attribute is set to `false`.
        is_idle: bool
    }
}

impl<T> Instrumentation<T> {
    pub fn new(inner: T, span: &'static SpanMetadata) -> Self {
        Instrumentation {
            inner,
            span,
            is_idle: false,
        }
    }
}

impl<T: Future> Future for Instrumentation<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let output = this.inner.poll(cx);

        if output.is_pending() && !*this.is_idle {
            on_end_scope(this.span);

            *this.is_idle = true;
        }

        if output.is_ready() {
            on_begin_scope(this.span);

            *this.is_idle = false;
        }

        output
    }
}
