use crate::DualTime;

#[derive(Debug)]
pub struct EventBlock<Q> {
    pub stream_id: String,
    pub begin: DualTime,
    pub events: Q,
    pub end: Option<DualTime>,
}

impl<Q> EventBlock<Q>
where
    Q: lgn_tracing_transit::HeterogeneousQueue,
{
    pub fn close(&mut self) {
        self.end = Some(DualTime::now());
    }
}

pub trait TelemetryBlock {
    type Queue;
    fn new(buffer_size: usize, stream_id: String) -> Self;
    fn len_bytes(&self) -> usize;
    fn nb_objects(&self) -> usize;
    fn events_mut(&mut self) -> &mut Self::Queue;
}

impl<Q> TelemetryBlock for EventBlock<Q>
where
    Q: lgn_tracing_transit::HeterogeneousQueue,
{
    type Queue = Q;
    fn new(buffer_size: usize, stream_id: String) -> Self {
        Self {
            stream_id,
            begin: DualTime::now(),
            events: Q::new(buffer_size),
            end: None,
        }
    }

    fn len_bytes(&self) -> usize {
        self.events.len_bytes()
    }

    fn nb_objects(&self) -> usize {
        self.events.nb_objects()
    }

    fn events_mut(&mut self) -> &mut Self::Queue {
        &mut self.events
    }
}
