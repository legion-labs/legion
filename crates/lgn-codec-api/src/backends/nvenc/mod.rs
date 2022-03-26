use self::nv_encoder::NvEncEncoder;

use crate::{encoder_work_queue::EncoderWorkItem, VideoProcessor};

mod cuda;
pub mod nv_encoder;

mod loader;

pub use cuda::{CuContext, CuDevice};
pub use loader::{CudaApi, NvEncApi};

use super::EncoderConfig;

pub struct NvEncEncoderWrapper {
    _thread: Option<std::thread::JoinHandle<()>>,
    encoder: NvEncEncoder,
}

impl VideoProcessor for NvEncEncoderWrapper {
    type Input = EncoderWorkItem;
    type Output = Vec<u8>;
    type Config = EncoderConfig;

    fn submit_input(&self, input: &Self::Input) -> Result<(), crate::Error> {
        self.encoder.encode_frame(input);
        Ok(())
    }

    fn query_output(&self) -> Result<Self::Output, crate::Error> {
        Ok(self.encoder.process_encoded_data())
    }

    fn new(mut config: Self::Config) -> Option<Self> {
        if let Some(encoder) = NvEncEncoder::new() {
            encoder.initialize_encoder();

            let encoder_for_closure = encoder.clone();
            Some(Self {
                _thread: Some(std::thread::spawn(move || {
                    NvEncEncoder::encoder_loop(&mut config.work_queue, &encoder_for_closure);
                })),
                encoder,
            })
        } else {
            None
        }
    }
}
