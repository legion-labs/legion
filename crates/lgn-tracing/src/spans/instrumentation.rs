//! Async instrumentation

#![allow(clippy::use_self)]

use std::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};

use pin_project::pin_project;

use crate::dispatch::{on_begin_scope, on_end_scope};

use super::SpanMetadata;

#[pin_project]
pub struct Instrumentation<'a, T> {
    #[pin]
    inner: T,
    span: &'static SpanMetadata,
    is_idle: &'a AtomicBool,
}

impl<'a, T> Instrumentation<'a, T> {
    pub fn new(inner: T, span: &'static SpanMetadata, is_idle: &'a AtomicBool) -> Self {
        Instrumentation {
            inner,
            span,
            is_idle,
        }
    }

    pub fn begin(&self) {
        on_begin_scope(self.span);
    }

    pub fn end(&self) {
        on_end_scope(self.span);
    }
}

impl<'a, T: Future> Future for Instrumentation<'a, T> {
    type Output = T::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();

        let is_idle = this.is_idle.load(Ordering::Relaxed);

        if !is_idle {
            on_end_scope(this.span);

            this.is_idle.store(true, Ordering::Relaxed);
        }

        let output = this.inner.poll(cx);

        if output.is_ready() && is_idle {
            on_begin_scope(this.span);

            this.is_idle.store(false, Ordering::Relaxed);
        }

        output
    }
}
