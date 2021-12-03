// BEGIN - Legion Labs lints v0.6
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
    clippy::if_not_else,
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
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow()]

use std::{path::PathBuf, str::FromStr};

use clap::{AppSettings, Arg, SubCommand};
use legion_content_store::ContentStoreAddr;
use legion_data_build::DataBuildOptions;
use legion_data_compiler::{Locale, Platform, Target};
use legion_data_offline::ResourcePathId;
use legion_data_runtime::resource_type_id_tuple;

const ARG_NAME_RESOURCE_PATH: &str = "resource";
const ARG_NAME_BUILDINDEX: &str = "buildindex";
const ARG_NAME_PROJECT: &str = "project";
const ARG_NAME_CAS: &str = "cas";
const ARG_NAME_MANIFEST: &str = "manifest";
const ARG_RUNTIME_FLAG: &str = "rt";
const ARG_NAME_TARGET: &str = "target";
const ARG_NAME_PLATFORM: &str = "platform";
const ARG_NAME_LOCALE: &str = "locale";

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), String> {
    let matches = clap::App::new("Data Build")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("Data Build CLI")
        .subcommand(
            SubCommand::with_name("create")
                .about("Create build index at a specified location")
                .arg(
                    Arg::with_name(ARG_NAME_BUILDINDEX)
                        .required(true)
                        .help("New build index path."),
                )
                .arg(
                    Arg::with_name(ARG_NAME_PROJECT)
                        .required(true)
                        .takes_value(true)
                        .long(ARG_NAME_PROJECT)
                        .help("Source project path."),
                ),
        )
        .subcommand(
            SubCommand::with_name("compile")
                .about("Compile input resource.")
                .arg(
                    Arg::with_name(ARG_NAME_RESOURCE_PATH)
                        .required(true)
                        .help("Path in build graph to compile."),
                )
                .arg(
                    Arg::with_name(ARG_NAME_BUILDINDEX)
                        .takes_value(true)
                        .required(true)
                        .long(ARG_NAME_BUILDINDEX)
                        .help("Build index file."),
                )
                .arg(
                    Arg::with_name(ARG_NAME_CAS)
                        .takes_value(true)
                        .long(ARG_NAME_CAS)
                        .required(true)
                        .multiple(true)
                        .help("Compiled Asset Store addresses where assets will be output."),
                )
                .arg(
                    Arg::with_name(ARG_NAME_MANIFEST)
                        .takes_value(true)
                        .long(ARG_NAME_MANIFEST)
                        .help("Manifest file path."),
                )
                .arg(
                    Arg::with_name(ARG_RUNTIME_FLAG)
                        .long(ARG_RUNTIME_FLAG)
                        .help(
                        "Accept ResourceId as the compilation input and output a runtime manifest.",
                    ),
                )
                .arg(
                    Arg::with_name(ARG_NAME_TARGET)
                        .required(true)
                        .takes_value(true)
                        .long(ARG_NAME_TARGET)
                        .help("Build target (Game, Server, etc)."),
                )
                .arg(
                    Arg::with_name(ARG_NAME_PLATFORM)
                        .required(true)
                        .takes_value(true)
                        .long(ARG_NAME_PLATFORM)
                        .help("Build platform (Windows, Unix, etc)"),
                )
                .arg(
                    Arg::with_name(ARG_NAME_LOCALE)
                        .required(true)
                        .takes_value(true)
                        .long(ARG_NAME_LOCALE)
                        .help("Build localization (en, fr, etc)"),
                ),
        )
        .get_matches();

    if let ("create", Some(cmd_args)) = matches.subcommand() {
        let buildindex_path = PathBuf::from(cmd_args.value_of(ARG_NAME_BUILDINDEX).unwrap());
        let project_path = PathBuf::from(cmd_args.value_of(ARG_NAME_PROJECT).unwrap());

        let mut build = DataBuildOptions::new(&buildindex_path)
            .content_store(&ContentStoreAddr::from("."))
            .create(project_path)
            .map_err(|e| format!("failed creating build index {}", e))?;

        if let Err(e) = build.source_pull() {
            eprintln!("Source Pull failed with '{}'", e);
            let _res = std::fs::remove_file(buildindex_path);
        }
    } else if let ("compile", Some(cmd_args)) = matches.subcommand() {
        let derived = cmd_args.value_of(ARG_NAME_RESOURCE_PATH).unwrap();
        let target = cmd_args.value_of(ARG_NAME_TARGET).unwrap();
        let platform = cmd_args.value_of(ARG_NAME_PLATFORM).unwrap();
        let locale = cmd_args.value_of(ARG_NAME_LOCALE).unwrap();
        let target =
            Target::from_str(target).map_err(|_e| format!("Invalid Target '{}'", target))?;
        let platform = Platform::from_str(platform)
            .map_err(|_e| format!("Invalid Platform '{}'", platform))?;
        let locale = Locale::new(locale);
        let content_store_path = ContentStoreAddr::from(cmd_args.value_of(ARG_NAME_CAS).unwrap());
        let buildindex_dir = PathBuf::from(cmd_args.value_of(ARG_NAME_BUILDINDEX).unwrap());
        let runtime_flag = cmd_args.is_present(ARG_RUNTIME_FLAG);

        let manifest_file = {
            if let Some(manifest) = cmd_args.value_of(ARG_NAME_MANIFEST) {
                let manifest_file = PathBuf::from_str(manifest)
                    .map_err(|_e| format!("Invalid Manifest name '{}'", manifest))?;
                Some(manifest_file)
            } else {
                None
            }
        };

        let mut config = DataBuildOptions::new(buildindex_dir);
        config.content_store(&content_store_path);
        if let Ok(cwd) = std::env::current_dir() {
            config.compiler_dir(cwd);
        }
        if let Some(mut exe_dir) = std::env::args().next().map(|s| PathBuf::from(&s)) {
            if exe_dir.pop() && exe_dir.is_dir() {
                config.compiler_dir(exe_dir);
            }
        }

        let mut build = config
            .open()
            .map_err(|e| format!("Failed to open build index: '{}'", e))?;

        let derived = {
            if runtime_flag {
                let id = resource_type_id_tuple::from_str(derived)
                    .map_err(|_e| format!("Invalid Resource (ResourceId) '{}'", derived))?;
                build.lookup_pathid(id).ok_or(format!(
                    "Cannot resolve ResourceId to ResourcePathId: '{}'",
                    resource_type_id_tuple::to_string(id)
                ))?
            } else {
                ResourcePathId::from_str(derived)
                    .map_err(|_e| format!("Invalid Resource (ResourcePathId) '{}'", derived))?
            }
        };

        //
        // for now, each time we build we make sure we have a fresh input data indexed
        // by doing a source_pull. this should most likely be executed only on demand.
        //
        build
            .source_pull()
            .map_err(|e| format!("Source Pull Failed: '{}'", e))?;

        let output = build
            .compile(derived, manifest_file, target, platform, &locale)
            .map_err(|e| format!("Compilation Failed: '{}'", e))?;

        if runtime_flag {
            let output = output.into_rt_manifest(|_| true);
            let output = serde_json::to_string(&output).unwrap();
            println!("{}", output);
        } else {
            let output = serde_json::to_string(&output).unwrap();
            println!("{}", output);
        }
    }
    Ok(())
}
