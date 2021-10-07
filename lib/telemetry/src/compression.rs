use anyhow::*;
use std::io::Write;

pub fn compress(src: &[u8]) -> Result<Vec<u8>> {
    let mut compressed = Vec::new();
    let mut encoder = lz4::EncoderBuilder::new()
        .level(10)
        .build(&mut compressed)
        .with_context(|| "allocating lz4 encoder")?;
    let _size = encoder
        .write(src)
        .with_context(|| "writing to lz4 encoder")?;
    let (_writer, _res) = encoder.finish();
    _res.with_context(|| "closing lz4 encoder")?;
    Ok(compressed)
}
