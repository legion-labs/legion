use crate::DualTime;

#[derive(Debug)]
pub struct EventBlock<Q> {
    pub stream_id: String,
    pub begin: DualTime,
    pub events: Q,
    pub end: Option<DualTime>,
}

impl<Q> EventBlock<Q> {
    pub fn new(event_queue: Q, stream_id: String) -> Self {
        Self {
            stream_id,
            begin: DualTime::now(),
            events: event_queue,
            end: None,
        }
    }

    pub fn close(&mut self) {
        self.end = Some(DualTime::now());
    }
}
