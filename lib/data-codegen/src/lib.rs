//! Code generation module for Data Model

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
mod component_codegen;
mod reflection;
mod reflection_codegen;
mod resource_codegen;
mod runtime_codegen;

use std::error::Error;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use quote::{format_ident, ToTokens};
use reflection::DataContainerMetaInfo;

/// Type of Generation
#[derive(PartialEq, Clone, Copy)]
pub enum GenerationType {
    /// Generate code for Offline (tools, editor)
    OfflineFormat,
    /// Generate code for Runtime
    RuntimeFormat,
}

/// Directory Code Generator (called from Build Scripts)
/// # Errors
pub fn generate_for_directory(directory: &std::path::Path) -> Result<(), Box<dyn Error>> {
    let codegen_dir = directory.parent().unwrap().join("codegen");
    std::fs::create_dir_all(&codegen_dir)?;

    [GenerationType::OfflineFormat, GenerationType::RuntimeFormat]
        .into_iter()
        .try_for_each(|gen_type| -> Result<(), Box<dyn Error>> {
            // Create a mod.rs per gentype
            let mod_path = if gen_type == GenerationType::OfflineFormat {
                codegen_dir.join("offline")
            } else {
                codegen_dir.join("runtime")
            };
            std::fs::create_dir_all(&mod_path)?;
            let mut mod_file = std::fs::File::create(&mod_path.join("mod.rs"))?;

            let mut processed_types = Vec::<DataContainerMetaInfo>::new();

            // Process all the .rs inside the directory
            let mut paths = std::fs::read_dir(directory)?
                .map(|result| result.map(|entry| entry.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()?;
            // Since the order in which read_dir returns entries is platform+filesystem
            // dependent, sort to guarantee determinism
            paths.sort();
            for path in paths {
                let filename = path.file_name().unwrap().to_ascii_lowercase();

                if let Some(ext) = path.extension() {
                    if ext.to_ascii_lowercase() == "rs" && filename != "build.rs" {
                        let types = generate_data_container_code(&path, &mod_path, gen_type)?;
                        processed_types.extend(types);

                        writeln!(
                            mod_file,
                            r#"#[path = "../{}/{}"]"#,
                            if gen_type == GenerationType::OfflineFormat {
                                "offline"
                            } else {
                                "runtime"
                            },
                            filename.to_str().unwrap()
                        )?;

                        let sub_mod_name = filename.to_str().unwrap().strip_suffix(".rs").unwrap();
                        writeln!(mod_file, "mod {};", sub_mod_name)?;
                        writeln!(mod_file, "pub use {}::*;\n", sub_mod_name)?;
                    }
                }
            }

            // Add Registration/Loader code
            let out_token = if gen_type == GenerationType::OfflineFormat {
                resource_codegen::generate_registration_code(&processed_types)
            } else {
                runtime_codegen::generate_registration_code(&processed_types)
            };
            mod_file.write_all(out_token.to_string().as_bytes())?;

            mod_file.flush()?;
            Command::new("rustfmt")
                .args(&[mod_path.join("mod.rs").as_os_str()])
                .status()?;

            Ok(())
        })?;

    Ok(())
}

/// Default Code Generator (called from Build Scripts)
/// # Errors
pub fn generate_data_container_code(
    source_path: &std::path::Path,
    out_dir: &std::path::Path,
    gen_type: GenerationType,
) -> Result<Vec<DataContainerMetaInfo>, Box<dyn Error>> {
    let src = std::fs::read_to_string(source_path).expect("Read file");
    let ast = syn::parse_file(&src).expect("Unable to parse file");

    let source_file_name = source_path.file_name().unwrap().to_ascii_lowercase();
    let gen_path = out_dir.join(&source_file_name);

    let mut gen_file = std::fs::File::create(&gen_path)?;

    // Write 'uses' from definition
    ast.items
        .iter()
        .filter_map(|item| match &item {
            syn::Item::Use(uses) => Some(uses.to_token_stream()),
            _ => None,
        })
        .try_for_each(|ts| gen_file.write_all(ts.to_string().as_bytes()))?;

    // Gather info about the structs
    let structs: Vec<DataContainerMetaInfo> = ast
        .items
        .iter()
        .filter_map(|item| match &item {
            syn::Item::Struct(item_struct) => reflection::get_data_container_info(item_struct).ok(),
            _ => None,
        })
        .collect();

    // Generate struct code
    structs
        .iter()
        .enumerate()
        .try_for_each(|(index, meta_info)| {
            let out_token = reflection_codegen::generate_reflection(meta_info, gen_type);
            gen_file.write_all(out_token.to_string().as_bytes())?;

            // generate component traits
            if meta_info.is_component {
                gen_file.write_all(
                    component_codegen::generate_component(meta_info, gen_type)
                        .to_string()
                        .as_bytes(),
                )?;
            }
            // generate resources traits
            if meta_info.is_resource {
                let token_stream = if gen_type == GenerationType::OfflineFormat {
                    resource_codegen::generate(meta_info, index == 0)
                } else {
                    runtime_codegen::generate(meta_info, index == 0)
                };
                gen_file.write_all(token_stream.to_string().as_bytes())?;
            }
            writeln!(gen_file)
        })?;

    gen_file.flush()?;

    Command::new("rustfmt")
        .args(&[gen_path.as_os_str()])
        .status()?;

    Ok(structs)
}

fn extract_crate_name(path: &Path) -> syn::Ident {
    let crate_name = std::fs::read_to_string(path.join("Cargo.toml"))
        .ok()
        .and_then(|config_toml| config_toml.parse::<toml::Value>().ok())
        .and_then(|value| value.try_into::<toml::value::Table>().ok())
        .and_then(|table| {
            if let Some(section) = table.get("package") {
                if let Some(toml::Value::String(name)) = section.get("name") {
                    return Some(name.replace("-", "_"));
                }
            }
            None
        })
        .unwrap_or_else(|| panic!("Cannot find inside Cargo.toml {:?}", &path));

    format_ident!("{}", crate_name)
}

/// Default Code Generator (called from Build Scripts)
/// # Errors
pub fn generate_data_compiler_code(
    source_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let src = std::fs::read_to_string(source_path).expect("Read file");
    let ast = syn::parse_file(&src).expect("Unable to parse file");

    let package_path = source_path.parent().unwrap().parent().unwrap();

    // Extract name of the runtime and offline crate from their Cargo.toml files
    let crate_name = extract_crate_name(package_path);

    for item in &ast.items {
        if let syn::Item::Struct(item_struct) = &item {
            if let Ok(meta_info) = reflection::get_data_container_info(item_struct) {
                if meta_info.is_resource {
                    let gen_path =
                        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join(
                            Path::new(&format!("compiler_{}.rs", &meta_info.name.to_lowercase())),
                        );

                    let mut gen_file = std::fs::File::create(&gen_path)?;

                    let out_token = compiler_codegen::generate(&meta_info, &crate_name);
                    gen_file.write_all(out_token.to_string().as_bytes())?;
                    gen_file.flush()?;

                    Command::new("rustfmt")
                        .args(&[gen_path.as_os_str()])
                        .status()?;
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
macro_rules! compiler_container_gen {
    ( $x:expr ) => {
        let mut data_path = std::path::Path::new(&env!("CARGO_MANIFEST_DIR"));
        let data_path = data_path.join($x);
        lgn_data_codegen::generate_data_compiler_code(&data_path).expect("Compiler codegen failed");
        println!("cargo:rerun-if-changed={}", data_path.display());
    };
}
