use lgn_data_runtime::Resource;

/// A simple API to access resources that contain raw data.
pub trait RawContent: Resource {
    ///
    fn set_raw_content(&mut self, data: &[u8]);
}
