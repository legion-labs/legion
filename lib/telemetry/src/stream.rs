use crate::{EncodedBlock, StreamInfo};

pub trait Stream {
    fn get_stream_info(&self) -> StreamInfo;
    fn get_stream_id(&self) -> String;
}

pub trait StreamBlock {
    fn encode(&self) -> EncodedBlock;
}
