use self::nv_encoder_session::NvEncoderSession;

use crate::stream_encoder::{EncoderWorkItem, StreamEncoder};

mod cuda;
pub mod nv_encoder;
pub mod nv_encoder_session;

mod loader;

pub use cuda::{CuContext, CuDevice};
pub use loader::{CudaApi, NvEncApi};

pub struct StreamEncoderSesssion {
    session: NvEncoderSession,
}

impl StreamEncoderSesssion {
    pub fn submit_input(&mut self, input: &EncoderWorkItem) {
        self.session.encode_frame(input);
    }

    pub fn query_output(&mut self) -> Vec<u8> {
        self.session.process_encoded_data()
    }

    pub fn new(stream_encoder: &StreamEncoder) -> Option<Self> {
        stream_encoder.hw_encoder().and_then(|hw_encoder| {
            NvEncoderSession::new(&hw_encoder).map(|session| Self { session })
        })
    }
}
