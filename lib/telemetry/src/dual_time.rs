use core::arch::x86_64::_rdtsc;

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct DualTime {
    pub ticks: i64,
    pub time: DateTime<Utc>,
}

#[allow(clippy::cast_possible_wrap)]
pub fn now() -> i64 {
    //_rdtsc does not wait for previous instructions to be retired
    // we could use __rdtscp if we needed more precision at the cost of slightly higher overhead
    unsafe { _rdtsc() as i64 }
}

impl DualTime {
    pub fn now() -> Self {
        Self {
            ticks: crate::now(),
            time: Utc::now(),
        }
    }
}
