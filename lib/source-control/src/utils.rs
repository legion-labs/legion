use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

pub enum SearchResult<T, E> {
    Ok(T),
    Err(E),
    None,
}

pub fn hash_string(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:X}", hasher.finalize())
}

pub fn create_parent_directory(path: &Path) -> Result<()> {
    let parent_dir = path.parent().unwrap();

    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir).context(format!(
            "error creating directory: {}",
            parent_dir.display()
        ))
    } else {
        Ok(())
    }
}

pub fn write_file(path: &Path, contents: &[u8]) -> Result<()> {
    create_parent_directory(path)?;

    let mut file =
        fs::File::create(path).context(format!("error creating file: {}", path.display()))?;

    file.write_all(contents)
        .context(format!("error writing: {}", path.display()))
}

pub fn write_new_file(path: &Path, contents: &[u8]) -> Result<()> {
    create_parent_directory(path)?;

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .context(format!("error writing file: {}", path.display()))?;

    file.write_all(contents)
        .context(format!("error writing: {}", path.display()))
}

pub fn read_text_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).context(format!("error reading file: {}", path.display()))
}

pub fn read_bin_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).context(format!("error reading file {}", path.display()))
}

pub fn make_path_absolute(p: &Path) -> PathBuf {
    //fs::canonicalize is a trap - it generates crazy unusable "extended length"
    // paths
    if p.is_absolute() {
        PathBuf::from(path_clean::clean(p.to_str().unwrap()))
    } else {
        PathBuf::from(path_clean::clean(
            std::env::current_dir().unwrap().join(p).to_str().unwrap(),
        ))
    }
}

pub fn path_relative_to(p: &Path, base: &Path) -> Result<PathBuf> {
    p.strip_prefix(base).map(Path::to_path_buf).context(format!(
        "error stripping prefix: {} is not relative to {}",
        p.display(),
        base.display()
    ))
}

pub fn make_canonical_relative_path(
    workspace_root: &Path,
    path_specified: &Path,
) -> Result<String> {
    let abs_path = make_path_absolute(path_specified);
    let relative_path = path_relative_to(&abs_path, workspace_root)?;
    let canonical_relative_path = relative_path.to_str().unwrap().replace("\\", "/");
    Ok(canonical_relative_path)
}

pub fn make_file_read_only(file_path: &Path, readonly: bool) -> Result<()> {
    let meta = fs::metadata(&file_path)
        .context(format!("error reading metadata: {}", file_path.display()))?;

    let mut permissions = meta.permissions();
    permissions.set_readonly(readonly);

    fs::set_permissions(&file_path, permissions).context(format!(
        "error setting permissions: {}",
        file_path.display()
    ))
}

pub fn lz4_compress_to_file(file_path: &Path, contents: &[u8]) -> Result<()> {
    create_parent_directory(file_path)?;

    let output_file = std::fs::File::create(file_path)
        .context(format!("error creating file: {}", file_path.display()))?;

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

pub fn lz4_read(compressed: &Path) -> Result<String> {
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

pub fn lz4_read_bin(compressed: &Path) -> Result<Vec<u8>> {
    let input_file = std::fs::File::open(compressed)
        .context(format!("error opening file: {}", compressed.display()))?;
    let mut decoder = lz4::Decoder::new(input_file)
        .context(format!("error reading file: {}", compressed.display()))?;
    let mut res = Vec::new();
    decoder
        .read_to_end(&mut res)
        .context(format!("error reading file: {}", compressed.display()))?;

    Ok(res)
}

pub fn lz4_decompress(compressed: &Path, destination: &Path) -> Result<()> {
    create_parent_directory(destination)?;

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
