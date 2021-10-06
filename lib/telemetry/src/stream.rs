use crate::{EncodedBlock, StreamInfo};
use anyhow::Result;

pub trait Stream {
    fn get_stream_info(&self) -> StreamInfo;
    fn get_stream_id(&self) -> String;
}

pub trait StreamBlock {
    fn encode(&self) -> Result<EncodedBlock>;
}
