use std::{sync::atomic::AtomicU32, u32};

use crate::{CpuBuffer, Error, GpuImage, VideoProcessor};

/// Null Encoder Config
#[derive(Debug)]
pub struct NullEncoderConfig {
    /// queue size used to simulate a hw encoder queue
    pub queue_size: u32,
}

/// Null Encoder, used in testing or as as dropping
/// replacement of a hw encoder in case you want to keep
/// the same code even if a given hw encoder fails to initialize
/// even if the new function returns an Option, it is guaranteed to work
#[derive(Debug)]
pub struct NullEncoder {
    queue_size: u32,
    queue_count: AtomicU32,
}

impl VideoProcessor for NullEncoder {
    type Input = GpuImage;
    type Output = CpuBuffer;
    type Config = NullEncoderConfig;

    fn submit_input(&self, _input: &Self::Input) -> Result<(), crate::Error> {
        let current_val = self.queue_count.load(std::sync::atomic::Ordering::SeqCst);
        if current_val < self.queue_size {
            self.queue_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        } else {
            Err(Error::BufferFull)
        }
    }

    fn query_output(&self) -> Result<Self::Output, crate::Error> {
        let current_val = self.queue_count.load(std::sync::atomic::Ordering::SeqCst);
        if current_val == 0 {
            Err(crate::Error::NeedInputs)
        } else {
            self.queue_count
                .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            Ok(CpuBuffer(Vec::new()))
        }
    }

    fn new(config: Self::Config) -> Option<Self> {
        Some(Self {
            queue_size: config.queue_size,
            queue_count: AtomicU32::new(0),
        })
    }
}

impl Default for NullEncoderConfig {
    fn default() -> Self {
        Self { queue_size: 10 }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use super::{NullEncoder, NullEncoderConfig};
    use crate::{GpuImage, VideoProcessor};

    #[test]
    fn null_encoder() {
        let count = 10000;
        let encoder = Arc::new(
            NullEncoder::new(NullEncoderConfig {
                queue_size: u32::MAX,
            })
            .expect("Null encoder should never fail"),
        );

        let thread_encoder = encoder.clone();
        let thread_handle = thread::spawn(move || {
            for _ in 0..count {
                while thread_encoder.query_output().is_err() {}
            }
        });

        for _ in 0..count {
            encoder
                .submit_input(&GpuImage::Vulkan(ash::vk::Image::null()))
                .expect("Submit should never fail since queue size is higher than submitted input");
        }

        thread_handle
            .join()
            .expect("Thread should terminate gracefully");

        assert_eq!(
            encoder
                .queue_count
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
    }
}
