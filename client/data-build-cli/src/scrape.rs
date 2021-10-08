//! Data scraping utility.
//!
//! Provides functionalities that help diagnose project's data.
//!
//! # Resource Type Tool - `rty`
//!
//! Various commands to help identify resource types known under a specified sourcecode directory.
//!
//! ### List resource types
//!
//! List all **resource types** found in code under a specified directory:
//!
//! ```text
//! $ data-scrape rty lib\ list
//! runtime_texture = f9c9670d
//! runtime_material = 669eb7d0
//! offline_texture = 74dc0e53
//! psd = 13b5a84e
//! ```
//!
//! ### Encode `Resource::TYPENAME`
//!
//! Show the hashed value of a given resource type name (in code under a specified directory).
//!
//! ```text
//! $ data-scrape rty lib\ encode psd
//! psd = 13b5a84e
//! ```
//!
//! ### Decode `Resource::TYPE`
//!
//! Show a human readable name of a given resource hash (in code under a specified directory).
//!
//! ```text
//! $ data-scrape rty lib\ decode runtime_texture
//! runtime_texture = f9c9670d
//! ```

// BEGIN - Legion Labs lints v0.5
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs standard lints v0.5
// crate-specific exceptions:
#![allow()]

use std::path::{Path, PathBuf};

use clap::{AppSettings, Arg, SubCommand};
use legion_data_runtime::ResourceType;

fn main() {
    let matches = clap::App::new("Data Scraper")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("Data Build CLI")
        .subcommand(
            SubCommand::with_name("rty")
                .about("Parse code for ResourceType information")
                .arg(Arg::with_name("path").help("Path in to code root."))
                .subcommand(SubCommand::with_name("list"))
                .subcommand(
                    SubCommand::with_name("encode")
                        .arg(
                            Arg::with_name("name")
                                .help("Human readable resource name - Resource::TYPENAME")
                                .required(true),
                        )
                        .about("Encodes human readable resource name to hash value."),
                )
                .subcommand(
                    SubCommand::with_name("decode")
                        .arg(
                            Arg::with_name("ty")
                                .help("ResourceType hash - Resource::TYPE")
                                .required(true),
                        )
                        .about("Decodes hash value of resource type to human readable name"),
                ),
        )
        .get_matches();

    if let ("rty", Some(cmd_args)) = matches.subcommand() {
        let code_dir = cmd_args
            .value_of("path")
            .map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

        match cmd_args.subcommand() {
            ("list", _) => {
                for (name, ty) in ResourceTypeIterator::new(find_files(&code_dir, &["rs"])) {
                    println!("{} = {}", name, ty);
                }
            }
            ("encode", Some(cmd_args)) => {
                let searched_name = cmd_args.value_of("name").unwrap();

                if let Some((name, ty)) = ResourceTypeIterator::new(find_files(&code_dir, &["rs"]))
                    .find(|(name, _)| name == searched_name)
                {
                    println!("{} = {}", name, ty);
                }
            }
            ("decode", Some(cmd_args)) => {
                let searched_ty = cmd_args.value_of("ty").unwrap();
                let searched_ty =
                    ResourceType::from_raw(u32::from_str_radix(searched_ty, 16).unwrap());
                if let Some((name, ty)) = ResourceTypeIterator::new(find_files(&code_dir, &["rs"]))
                    .find(|(_, ty)| ty == &searched_ty)
                {
                    println!("{} = {}", name, ty);
                }
            }
            _ => {
                println!("{}", cmd_args.usage());
            }
        }
    }
}

struct ResourceTypeIterator {
    files: Vec<PathBuf>,
    types: Vec<(String, ResourceType)>,
}

impl ResourceTypeIterator {
    fn new(files: Vec<PathBuf>) -> Self {
        Self {
            files,
            types: vec![],
        }
    }
}

impl Iterator for ResourceTypeIterator {
    type Item = (String, ResourceType);

    fn next(&mut self) -> Option<Self::Item> {
        while self.types.is_empty() {
            if let Some(next_file) = self.files.pop() {
                self.types = all_declared_resources(&next_file);
            } else {
                return None;
            }
        }
        if let Some(next_type) = self.types.pop() {
            return Some(next_type);
        }
        None
    }
}

// Recursively searches all directories in `dir` for files with `extensions`
fn find_files(dir: impl AsRef<Path>, extensions: &[&str]) -> Vec<PathBuf> {
    let dir = dir.as_ref();

    let mut files = vec![];

    for entry in dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.append(&mut find_files(&path, extensions));
        } else if let Some(ext) = path.extension().and_then(std::ffi::OsStr::to_str) {
            if extensions.contains(&ext) {
                files.push(path);
            }
        }
    }

    files
}

// recursively parses modules in search of resource attributes
fn find_resource_attribs(content: &[syn::Item]) -> Vec<(String, ResourceType)> {
    let get_resource_name = |a: &syn::ItemStruct| -> Option<(String, ResourceType)> {
        for a in &a.attrs {
            if a.path.segments.len() == 1 && a.path.segments[0].ident == "resource" {
                let arg: syn::LitStr = a.parse_args().unwrap();
                let arg = arg.value();

                let ty = ResourceType::new(arg.as_bytes());

                return Some((arg, ty));
            }
        }
        None
    };

    let mut types = vec![];
    for item in content {
        if let syn::Item::Struct(struc) = &item {
            if let Some((name, ty)) = get_resource_name(struc) {
                types.push((name, ty));
            }
        } else if let syn::Item::Mod(syn::ItemMod {
            content: Some((_, c)),
            ..
        }) = &item
        {
            types.extend(find_resource_attribs(c));
        }
    }
    types
}

// Finds all #[resource="name"] attributes in a file and returns (name, hashed name) tuple.
fn all_declared_resources(source: &Path) -> Vec<(String, ResourceType)> {
    let src = std::fs::read_to_string(&source).expect("Read file");
    let ast = syn::parse_file(&src).expect("Unable to parse file");
    find_resource_attribs(&ast.items)
}
