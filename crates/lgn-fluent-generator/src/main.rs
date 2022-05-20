//! Simple code generator for Fluent files.
//!
//! The CLI takes Fluent files as input and generates a typing file
//! (currently in TypeScript only).

use std::path::PathBuf;

use clap::Parser;
use error::Error;
use fluent_syntax::parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use types::{EntryDescription, RenderableTemplate};
use utils::read_files_from_glob;

use crate::{error::Result, types::TypeScriptTemplate};

mod error;
mod types;
mod utils;

#[derive(Debug, Parser)]
struct Args {
    /// An input glob that matches all the Fluent file to generate the types from
    #[clap(short, long)]
    input: PathBuf,
    /// The folder to output the generated type file to
    #[clap(short, long, default_value = ".")]
    out_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let ftl = read_files_from_glob(&args.input.to_string_lossy())?;

    let resource = parser::parse_runtime(ftl.as_str())
        .map_err(|(_partial_resource, errors)| Error::FluentParse(errors))?;

    let entry_descriptions = resource
        .body
        .par_iter()
        .filter_map(|entry| EntryDescription::try_from(entry).ok())
        .collect::<Vec<_>>();

    // TODO: Support more languages (Rust, Python?)
    TypeScriptTemplate::new(&entry_descriptions).render_to_dir(&args.out_dir)?;

    Ok(())
}
