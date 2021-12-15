use anyhow::{Context, Result};
use std::fs;
use std::io::prelude::*;
use std::path::Path;

pub(crate) fn lz4_compress_to_file(path: impl AsRef<Path>, contents: &[u8]) -> Result<()> {
    fs::create_dir_all(path.as_ref().parent().unwrap()).context(format!(
        "error creating directory: {}",
        path.as_ref().display()
    ))?;

    let output_file = std::fs::File::create(path.as_ref())
        .context(format!("error creating file: {}", path.as_ref().display()))?;

    let mut encoder = lz4::EncoderBuilder::new()
        .level(10)
        .build(output_file)
        .context("error creating encoder")?;

    encoder
        .write(contents)
        .context("error writing to encoder")?;

    encoder.finish().1.context("error finishing encoder")?;

    Ok(())
}

pub(crate) fn lz4_read(compressed: &Path) -> Result<String> {
    let input_file = std::fs::File::open(compressed)
        .context(format!("error opening file: {}", compressed.display()))?;

    let mut decoder = lz4::Decoder::new(input_file)
        .context(format!("error reading file: {}", compressed.display()))?;
    let mut res = String::new();

    decoder
        .read_to_string(&mut res)
        .context(format!("error reading file: {}", compressed.display()))?;

    Ok(res)
}

pub(crate) fn lz4_decompress(compressed: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination.parent().unwrap()).context(format!(
        "error creating directory: {}",
        destination.display()
    ))?;

    let input_file = std::fs::File::open(compressed)
        .context(format!("error opening file: {}", compressed.display()))?;

    let mut decoder = lz4::Decoder::new(input_file).context(format!(
        "error creating decoder from: {}",
        compressed.display()
    ))?;

    let mut output_file = std::fs::File::create(destination)
        .context(format!("error creating file: {}", destination.display()))?;

    std::io::copy(&mut decoder, &mut output_file).context(format!(
        "error decoding from {} to {}",
        compressed.display(),
        destination.display()
    ))?;

    Ok(())
}
