//! Procedural macros associated with runtime asset management module of data processing pipeline.

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
#![warn(missing_docs)]

mod editor_codegen;
mod json_codegen;
mod offline_codegen;
mod reflection;
mod resource_codegen;
mod runtime_codegen;

use quote::ToTokens;
use std::io::Write;
use std::process::Command;

/// Type of Generation
#[derive(PartialEq)]
pub enum GenerationType {
    /// Generate code for Offline (tools, editor)
    OfflineFormat,
    /// Generate code for Runtime
    RuntimeFormat,
}

/// Default Code Generator (called from Build Scripts)
/// # Errors
pub fn generate_data_container_code(
    source_path: &std::path::Path,
    gen_type: &GenerationType,
) -> Result<(), Box<dyn std::error::Error>> {
    let src = std::fs::read_to_string(source_path).expect("Read file");
    let ast = syn::parse_file(&src).expect("Unable to parse file");

    let gen_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .join(source_path.file_name().unwrap());

    let mut gen_file = std::fs::File::create(&gen_path)?;

    for item in &ast.items {
        if let syn::Item::Use(uses) = &item {
            gen_file.write_all(uses.to_token_stream().to_string().as_bytes())?;
        }
    }
    writeln!(gen_file)?;

    for item in &ast.items {
        if let syn::Item::Struct(item_struct) = &item {
            for a in &item_struct.attrs {
                if a.path.segments.len() == 1 && a.path.segments[0].ident == "data_container" {
                    if let Ok(meta_info) = reflection::get_data_container_info(item_struct) {
                        if *gen_type == GenerationType::OfflineFormat {
                            let out_token = offline_codegen::generate(&meta_info);
                            gen_file.write_all(out_token.to_string().as_bytes())?;

                            let out_token = json_codegen::generate(&meta_info);
                            gen_file.write_all(out_token.to_string().as_bytes())?;

                            let out_token = editor_codegen::generate(&meta_info);
                            gen_file.write_all(out_token.to_string().as_bytes())?;

                            let out_token = resource_codegen::generate(&meta_info);
                            gen_file.write_all(out_token.to_string().as_bytes())?;
                        } else {
                            let out_token = runtime_codegen::generate(&meta_info);
                            gen_file.write_all(out_token.to_string().as_bytes())?;
                        }
                    }
                }
            }
        }
    }

    gen_file.flush()?;

    Command::new("rustfmt")
        .args(&[gen_path.as_os_str()])
        .status()?;

    Ok(())
}

/// Helper function to be used in build.rs files to generate the proper
/// binding to be included by crates
///
/// # Errors
/// Returns `Err` if the data is format is not compliant
#[macro_export]
macro_rules! data_container_gen {
    ( $( $x:expr ),* ) => {
        {
            let package_path = env!("CARGO_MANIFEST_DIR").to_lowercase();

            let mut data_path = package_path.replace("_runtime", "_offline");
            $(
                data_path.push_str($x);
            )*

            if package_path.ends_with("_offline") {
                legion_data_codegen::generate_data_container_code(
                    std::path::Path::new(&data_path),
                    &legion_data_codegen::GenerationType::OfflineFormat,
                )?;
            } else if package_path.ends_with("_runtime") {
                legion_data_codegen::generate_data_container_code(
                    std::path::Path::new(&data_path),
                    &legion_data_codegen::GenerationType::RuntimeFormat,
                )?;
            }
            $(
                println!("cargo:rerun-if-changed={}", $x);
            )*
        }
    };
}
