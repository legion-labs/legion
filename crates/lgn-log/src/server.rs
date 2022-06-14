use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

use crate::api::log::{
    requests::LogEntriesRequest, responses::LogEntriesResponse, Api, TraceEvent,
};
use lgn_online::server::Result;

pub(crate) struct TraceEventDeque {
    trace_events: Arc<Mutex<VecDeque<TraceEvent>>>,
    capacity: usize,
}

impl Clone for TraceEventDeque {
    fn clone(&self) -> Self {
        TraceEventDeque {
            trace_events: Arc::clone(&self.trace_events),
            capacity: self.capacity,
        }
    }
}

impl Default for TraceEventDeque {
    fn default() -> Self {
        Self::with_capacity(5_000)
    }
}

impl TraceEventDeque {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            trace_events: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    pub(crate) fn push_back(&mut self, value: TraceEvent) {
        let mut trace_events = self.trace_events.lock().unwrap();

        if trace_events.len() >= self.capacity {
            let _trace_event = trace_events.pop_front();
        }

        trace_events.push_back(value);
    }

    fn to_vec(&self) -> Vec<TraceEvent> {
        let mut trace_events = self.trace_events.lock().unwrap();

        trace_events.drain(..).collect()
    }
}

/// The `api` implementation for the log server.
pub(crate) struct Server {
    trace_events: TraceEventDeque,
}

impl Server {
    pub(crate) fn new(trace_events: TraceEventDeque) -> Self {
        Self { trace_events }
    }
}

#[async_trait]
impl Api for Server {
    async fn log_entries(
        &self,
        _parts: http::request::Parts,
        _request: LogEntriesRequest,
    ) -> Result<LogEntriesResponse> {
        Ok(LogEntriesResponse::Status200(self.trace_events.to_vec()))
    }
}
