use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::Path,
};

use glob::glob;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::error::{Error, Result};

/// Read all files from a glob and concat their contents into a [`String`]
pub fn read_files_from_glob(pattern: &str) -> Result<String> {
    let ftl = glob(pattern)?
        .collect::<Vec<_>>()
        .into_par_iter()
        .try_fold(String::new, |ftl, path| {
            let content = read_to_string(path?)?;

            Ok::<_, Error>(ftl + &content)
        })
        .collect::<Result<String>>()?;

    Ok(ftl)
}

/// Write the [`String`] content into a file, if a file under the provided directory.
///
/// # Errors
///
/// If the provided directory already exists but is _not_ a directory, then an error occurs
pub fn write_string_content<P: AsRef<Path>, C: Into<String>>(out_dir: P, content: C) -> Result<()> {
    let out_dir = out_dir.as_ref();

    if !out_dir.exists() {
        create_dir_all(&out_dir)?;
    }

    if !out_dir.is_dir() {
        return Err(Error::OutDirNotDir);
    }

    let mut file = File::create(out_dir.join("fluent.d.ts"))?;

    file.write_all(content.into().as_bytes())?;

    Ok(())
}
