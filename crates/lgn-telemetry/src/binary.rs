use anyhow::{Context, Result};
use lgn_tracing_transit::read_any;
use std::io::{Read, Write};

/// Compress a buffer using lz4.
///
/// # Errors
///
/// This function will return an error if the compression fails.
pub fn compress(src: &[u8]) -> Result<Vec<u8>> {
    let mut compressed = Vec::new();
    let mut encoder = lz4::EncoderBuilder::new()
        .level(10)
        .build(&mut compressed)
        .with_context(|| "allocating lz4 encoder")?;
    let _size = encoder
        .write(src)
        .with_context(|| "writing to lz4 encoder")?;
    let (_writer, res) = encoder.finish();
    res.with_context(|| "closing lz4 encoder")?;
    Ok(compressed)
}

/// Decompress a buffer using lz4.
///
/// # Errors
///
/// This function will return an error if the decompression fails.
pub fn decompress(compressed: &[u8]) -> Result<Vec<u8>> {
    let mut decompressed = Vec::new();
    let mut decoder = lz4::Decoder::new(compressed).with_context(|| "allocating lz4 decoder")?;
    let _size = decoder
        .read_to_end(&mut decompressed)
        .with_context(|| "reading lz4-compressed buffer")?;
    let (_reader, res) = decoder.finish();
    res?;
    Ok(decompressed)
}

/// Reads a binary chunk from the given buffer based on an `u32` size.
///
/// # Errors
///
/// This function will return an error if the read fails.
#[allow(unsafe_code)]
pub fn read_binary_chunk(buffer: &[u8], cursor: &mut usize) -> Result<Vec<u8>> {
    unsafe {
        let chunk_size_bytes = read_any::<u32>(buffer.as_ptr().add(*cursor)) as usize;
        *cursor += std::mem::size_of::<u32>();
        let end = *cursor + chunk_size_bytes;
        if end > buffer.len() {
            anyhow::bail!("binary chunk larger than buffer");
        }
        let chunk_buffer = &buffer[(*cursor)..end];
        *cursor += chunk_size_bytes;
        Ok(chunk_buffer.to_vec())
    }
}
