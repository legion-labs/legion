use crate::StreamInfo;

pub trait Stream {
    fn get_stream_info(&self) -> StreamInfo;
}
