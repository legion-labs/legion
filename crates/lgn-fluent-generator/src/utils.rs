use std::fs::read_to_string;

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
