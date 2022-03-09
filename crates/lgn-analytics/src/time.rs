use lgn_telemetry_proto::telemetry::Process as ProcessInfo;

pub struct ConvertTicks {
    ts_offset: i64,
    inv_tsc_frequency: f64,
}

impl ConvertTicks {
    pub fn new(process: &ProcessInfo) -> Self {
        let ts_offset = process.start_ticks;
        let inv_tsc_frequency = get_process_tick_length_ms(process);
        Self {
            ts_offset,
            inv_tsc_frequency,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn get_time(&self, ts: i64) -> f64 {
        (ts - self.ts_offset) as f64 * self.inv_tsc_frequency
    }
}

pub fn get_process_tick_length_ms(process_info: &lgn_telemetry_proto::telemetry::Process) -> f64 {
    get_tsc_frequency_inverse_ms(process_info.tsc_frequency)
}

#[allow(clippy::cast_precision_loss)]
pub fn get_tsc_frequency_inverse_ms(tsc_frequency: u64) -> f64 {
    1000.0 / tsc_frequency as f64
}
