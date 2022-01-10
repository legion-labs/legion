//! Legion build utils
//! This crate is meant to provide helpers for code generation in the monorepo
//! We rely on code generation in multiple instances:
//! * Proto files that generate rust and javascript files
//! * Shader files definition that generate rust and hlsl
//! * Data containers that generate rust files
//!
//! There is 2 ways of handling generated files in the rust ecosystem:
//! * Relying on `OUT_DIR` environment variable to generate in place any
//!   necessary file. (tonic, windows api, ...)
//! * Generating the files in the repo and committing them to the repo.
//!   (rust-analyser, rusoto, ...)
//!
//! We can't generate files in the crate directory and not have them committed,
//! since we have to think about the case of an external dependency being
//! downloaded in the local immutable register.
//!
//! Advantages:
//! * Improves readability and UX of generated files (Go to definition works in
//!   VS Code, looking at code from github)
//! * Allows inclusion of generated files from other systems (Javasctript, hlsl
//!   in a uniform manner) since `OUT_DIR` is only know during the cargo build
//!   of a given crate.
//!
//! Drawbacks:
//! * Dummy conflict in generated code
//! * We lose the ability to modify some src files from the github web interface
//!   since you
//! * Confusion about non generated code and generated code (although mitigated
//!   by conventions)
//!
//! Restriction and rules:
//! * We can't have binary files checked in
//! * Modification of the generated files would not be allowed under any
//!   circumstances, the build machines fail if any change was detected
//! * Files whose generation ca be driven by features, or that are platform
//!   dependent would still use `OUT_DIR`.
//! * Other cases where the in repo generation doesn't bring much

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

use std::path::{Path, PathBuf};

use lgn_build_utils::{run_cmd, Context, Error, Language, Result};

/// Build proto files
///
/// # Errors
/// Returns a generation error or an IO error
pub fn build_protos(
    context: &Context,
    protos: &[impl AsRef<Path>],
    includes: &[impl AsRef<Path>],
    lang: Language,
) -> Result<()> {
    let out_dir = PathBuf::from(&context.codegen_out_dir());

    // Make sure the generated files don't show-up by default in Github's pull-requests.
    std::fs::write(
        out_dir.join(".gitattributes"),
        "** linguist-generated=true\n",
    )?;

    if lang.contains(Language::RUST) {
        tonic_build::configure()
            .out_dir(&out_dir)
            .compile(protos, includes)?;
    }
    if lang.contains(Language::TYPESCRIPT) {
        if Path::new("./package.json").exists() {
            {
                let lock = named_lock::NamedLock::create("pnpm_install").unwrap();
                let _guard = lock.lock().unwrap();
                run_cmd("pnpm", &["install"], ".")?;
            }
            let mut proto_plugin = PathBuf::from("./node_modules/.bin/protoc-gen-ts_proto");
            if cfg!(windows) {
                proto_plugin = PathBuf::from(".\\node_modules\\.bin\\protoc-gen-ts_proto.cmd");
            }
            if !proto_plugin.exists() {
                return Err(Error::Build(
                    "missing `ts-proto` in your package dependency".to_string(),
                ));
            }
            let plugin_arg = format!("--plugin=protoc-gen-ts_proto={}", proto_plugin.display());
            let proto_out_arg = format!("--ts_proto_out={}", out_dir.display());
            let mut args = vec![
                plugin_arg.as_str(),
                proto_out_arg.as_str(),
                "--ts_proto_opt=esModuleInterop=true",
                "--ts_proto_opt=outputClientImpl=grpc-web",
                "--ts_proto_opt=env=browser",
                "--ts_proto_opt=lowerCaseServiceMethods=true",
            ];
            let includes: Vec<_> = includes
                .iter()
                .map(|path| format!("--proto_path={}", path.as_ref().to_str().unwrap()))
                .collect();
            let mut include_args: Vec<_> =
                includes.iter().map(std::string::String::as_str).collect();
            args.append(&mut include_args);

            let mut protos_args: Vec<_> = protos
                .iter()
                .map(|path| path.as_ref().to_str().unwrap())
                .collect();
            args.append(&mut protos_args);
            run_cmd("protoc", &args, ".")?;
        } else {
            return Err(Error::Build(
                "a package.json file needs to be next to the build.rs".to_string(),
            ));
        }
    }

    for proto in protos {
        println!("cargo:rerun-if-changed={}", proto.as_ref().display());
    }

    Ok(())
}
