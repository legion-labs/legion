use std::sync::atomic::{AtomicU64, Ordering};

use lgn_tracing::{dispatch::init_thread_stream, imetric};

static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

pub(crate) struct RequestGuard {
    begin_ticks: i64,
}

impl RequestGuard {
    pub(crate) fn new() -> Self {
        init_thread_stream();
        let previous_count = REQUEST_COUNT.fetch_add(1, Ordering::SeqCst);
        imetric!("Request Count", "count", previous_count);

        let begin_ticks = lgn_tracing::now();
        Self { begin_ticks }
    }
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        let end_ticks = lgn_tracing::now();
        let duration = end_ticks - self.begin_ticks;
        imetric!("Request Duration", "ticks", duration as u64);
    }
}
