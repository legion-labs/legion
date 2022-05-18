#![allow(dead_code)]

pub enum EventType {
    Immediate,
    Duration,
    Both,
}

pub struct Event {
    event_type: EventType,
}

impl Event {
    #[inline]
    fn is_immediate_event() {
        /* */
    }

    #[inline]
    fn is_duration_event() {
        /* */
    }
}
