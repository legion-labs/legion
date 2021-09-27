use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct DualTime {
    pub ticks: u64,
    pub time: DateTime<Utc>,
}

impl DualTime {
    pub fn now() -> Self {
        Self {
            ticks: crate::now(),
            time: Utc::now(),
        }
    }
}
