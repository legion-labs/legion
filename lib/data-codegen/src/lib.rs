//! Procedural macros associated with runtime asset management module of data processing pipeline.

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
#![warn(missing_docs)]

mod compiler_codegen;
mod offline_codegen;
mod reflection;
mod resource_codegen;
mod runtime_codegen;

use std::path::Path;
use std::process::Command;
use std::{io::Write, path::PathBuf};

use quote::{format_ident, ToTokens};
use reflection::DataContainerMetaInfo;

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

    let mut processed_structs: Vec<DataContainerMetaInfo> = Vec::new();

    let mut add_uses = true;
    for item in &ast.items {
        if let syn::Item::Struct(item_struct) = &item {
            for a in &item_struct.attrs {
                if a.path.segments.len() == 1 && a.path.segments[0].ident == "data_container" {
                    if let Ok(meta_info) = reflection::get_data_container_info(item_struct) {
                        if *gen_type == GenerationType::OfflineFormat {
                            let out_token = offline_codegen::generate(&meta_info);
                            gen_file.write_all(out_token.to_string().as_bytes())?;

                            let out_token = resource_codegen::generate(&meta_info, add_uses);
                            gen_file.write_all(out_token.to_string().as_bytes())?;
                        } else {
                            let out_token = runtime_codegen::generate(&meta_info, add_uses);
                            gen_file.write_all(out_token.to_string().as_bytes())?;
                        }
                        add_uses = false;
                        processed_structs.push(meta_info);
                    }
                }
            }
        }
    }

    // Generate Loader and Processor Registration code
    let out_token = if *gen_type == GenerationType::OfflineFormat {
        resource_codegen::generate_registration_code(&processed_structs)
    } else {
        runtime_codegen::generate_registration_code(&processed_structs)
    };
    gen_file.write_all(out_token.to_string().as_bytes())?;

    gen_file.flush()?;

    Command::new("rustfmt")
        .args(&[gen_path.as_os_str()])
        .status()?;

    Ok(())
}

fn extract_crate_name(path: &Path) -> syn::Ident {
    let crate_name = std::fs::read_to_string(path.join("Cargo.toml"))
        .ok()
        .and_then(|config_toml| config_toml.parse::<toml::Value>().ok())
        .and_then(|value| value.try_into::<toml::value::Table>().ok())
        .and_then(|table| {
            if let Some(section) = table.get("package") {
                if let Some(toml::Value::String(name)) = section.get("name") {
                    return Some(name.clone());
                }
            }
            None
        })
        .unwrap_or_else(|| panic!("Cannot find inside Cargo.toml {:?}", &path));

    format_ident!("{}", crate_name)
}

/// Creates a path to a definition file relative to current crate's directory.
pub fn definition_path(path: impl AsRef<Path>) -> PathBuf {
    let package_path = env!("CARGO_MANIFEST_DIR").to_lowercase();
    std::path::Path::new(&package_path).join(path)
}

/// Default Code Generator (called from Build Scripts)
/// # Errors
pub fn generate_data_compiler_code(
    source_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let src = std::fs::read_to_string(source_path).expect("Read file");
    let ast = syn::parse_file(&src).expect("Unable to parse file");

    let package_path = source_path.parent().unwrap();

    // Extract name of the runtime and offline crate from their Cargo.toml files
    let offline_crate_name = extract_crate_name(package_path);
    let runtime_crate_name = extract_crate_name(Path::new(
        &String::from(package_path.to_str().unwrap()).replace("_offline", "_runtime"),
    ));

    for item in &ast.items {
        if let syn::Item::Struct(item_struct) = &item {
            for a in &item_struct.attrs {
                if a.path.segments.len() == 1 && a.path.segments[0].ident == "data_container" {
                    if let Ok(meta_info) = reflection::get_data_container_info(item_struct) {
                        let gen_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap())
                            .join(Path::new(&format!(
                                "compiler_{}.rs",
                                &meta_info.name.to_lowercase()
                            )));

                        let mut gen_file = std::fs::File::create(&gen_path)?;

                        let out_token = compiler_codegen::generate(
                            &meta_info,
                            &offline_crate_name,
                            &runtime_crate_name,
                        );
                        gen_file.write_all(out_token.to_string().as_bytes())?;
                        gen_file.flush()?;

                        Command::new("rustfmt")
                            .args(&[gen_path.as_os_str()])
                            .status()?;
                    }
                }
            }
        }
    }
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
        let package_path = env!("CARGO_MANIFEST_DIR").to_lowercase();
        $(
            let mut data_path = package_path.replace("_runtime", "_offline").replace("_compiler", "_offline");
            data_path.push_str($x);
            if package_path.ends_with("_offline") {
                legion_data_codegen::generate_data_container_code(
                    std::path::Path::new(&data_path),
                    &legion_data_codegen::GenerationType::OfflineFormat,
                ).expect("Offline data codegen failed");
            } else if package_path.ends_with("_runtime") {
                legion_data_codegen::generate_data_container_code(
                    std::path::Path::new(&data_path),
                    &legion_data_codegen::GenerationType::RuntimeFormat,
                ).expect("Runtime data codegen failed");
            } else if package_path.ends_with("_compiler") {
                legion_data_codegen::generate_data_compiler_code(std::path::Path::new(&data_path)
                ).expect("Compiler codegen failed");
            }
            println!("cargo:rerun-if-changed={}", data_path);
        )*
    };
}
