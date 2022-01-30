//! Code generation module for Data Model

// crate-specific lint exceptions:
#![warn(missing_docs)]

mod compiler_codegen;
mod component_codegen;
mod reflection;
mod reflection_codegen;
mod resource_codegen;
mod runtime_codegen;

use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{error::Error, io::Cursor};

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

impl GenerationType {
    /// Returns the name of the generation type
    pub fn name(self) -> &'static str {
        match self {
            GenerationType::OfflineFormat => "offline",
            GenerationType::RuntimeFormat => "runtime",
        }
    }
}

/// Directory Code Generator (called from Build Scripts)
/// # Errors
pub fn generate_for_directory(
    src_dir: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let src_dir = src_dir.as_ref();
    let out_dir = out_dir.as_ref();
    std::fs::create_dir_all(out_dir)?;
    let codegen_file_path = out_dir.join("data_def.rs");
    let mut codegen_file = std::fs::File::create(&codegen_file_path)?;
    writeln!(codegen_file, "// File is auto generated\n")?;
    [GenerationType::OfflineFormat, GenerationType::RuntimeFormat]
        .into_iter()
        .try_for_each(|gen_type| -> Result<(), Box<dyn Error>> {
            // Beginning of gentype  module
            writeln!(
                codegen_file,
                "\n///////////////////////////////////////////////////////////////////////////////"
            )?;
            writeln!(codegen_file, "// {} code generation", gen_type.name())?;
            writeln!(
                codegen_file,
                "///////////////////////////////////////////////////////////////////////////////\n"
            )?;
            writeln!(codegen_file, "#[allow(unused_imports)]")?;
            writeln!(codegen_file, "#[cfg(feature = \"{}\")]", gen_type.name())?;
            writeln!(codegen_file, "pub mod {} {{", gen_type.name())?;
            let mut processed_types = Vec::<DataContainerMetaInfo>::new();
            // Process all the .rs inside the directory
            let mut paths = std::fs::read_dir(src_dir)?
                .map(|result| result.map(|entry| entry.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()?;
            // Since the order in which read_dir returns entries is platform+filesystem
            // dependent, sort to guarantee determinism
            paths.sort();
            for path in paths {
                let filename = path.file_name().unwrap().to_ascii_lowercase();

                if let Some(ext) = path.extension() {
                    if ext.to_ascii_lowercase() == "rs" && filename != "build.rs" {
                        let (content, types) = generate_data_container_code(&path, gen_type)?;
                        processed_types.extend(types);
                        let sub_mod_name = filename.to_str().unwrap().strip_suffix(".rs").unwrap();
                        writeln!(codegen_file, "mod {} {{", sub_mod_name)?;
                        codegen_file.write_all(&content)?;
                        writeln!(codegen_file, "}}")?;
                        writeln!(codegen_file, "pub use {}::*;\n", sub_mod_name)?;
                    }
                }
            }

            // Add Registration/Loader code
            let out_token = if gen_type == GenerationType::OfflineFormat {
                resource_codegen::generate_registration_code(&processed_types)
            } else {
                runtime_codegen::generate_registration_code(&processed_types)
            };
            codegen_file.write_all(out_token.to_string().as_bytes())?;

            // End of the gentype module
            writeln!(codegen_file, "}}")?;
            Ok(())
        })?;

    codegen_file.flush()?;
    Command::new("rustfmt")
        .args(&[codegen_file_path.as_os_str()])
        .status()?;

    Ok(())
}

/// Default Code Generator (called from Build Scripts)
/// # Errors
pub fn generate_data_container_code(
    source_path: &std::path::Path,
    gen_type: GenerationType,
) -> Result<(Vec<u8>, Vec<DataContainerMetaInfo>), Box<dyn Error>> {
    let src = std::fs::read_to_string(source_path).expect("Read file");
    let ast = syn::parse_file(&src).expect("Unable to parse file");

    let mut cursor = Cursor::new(vec![]);

    // Write 'uses' from definition
    ast.items
        .iter()
        .filter_map(|item| match &item {
            syn::Item::Use(uses) => Some(uses.to_token_stream()),
            _ => None,
        })
        .try_for_each(|ts| cursor.write_all(ts.to_string().as_bytes()))?;

    // Gather info about the structs
    let structs: Vec<DataContainerMetaInfo> = ast
        .items
        .iter()
        .filter_map(|item| match &item {
            syn::Item::Struct(item_struct) => reflection::get_data_container_info(item_struct).ok(),
            _ => None,
        })
        .collect();

    // Write auto-added imports
    let imports = structs
        .iter()
        .flat_map(|s| {
            if gen_type == GenerationType::RuntimeFormat {
                s.runtime_imports()
            } else {
                s.offline_imports()
            }
        })
        .collect::<Vec<_>>();

    let imports = quote::quote! {
            #(use #imports;)*
    };
    cursor.write_all(imports.to_string().as_bytes())?;

    // Generate struct code
    structs.iter().try_for_each(|meta_info| {
        let out_token = reflection_codegen::generate_reflection(meta_info, gen_type);
        cursor.write_all(out_token.to_string().as_bytes())?;

        // generate component traits
        if meta_info.is_component {
            cursor.write_all(
                component_codegen::generate_component(meta_info, gen_type)
                    .to_string()
                    .as_bytes(),
            )?;
        }
        // generate resources traits
        if meta_info.is_resource {
            let token_stream = if gen_type == GenerationType::OfflineFormat {
                resource_codegen::generate(meta_info)
            } else {
                runtime_codegen::generate(meta_info)
            };
            cursor.write_all(token_stream.to_string().as_bytes())?;
        }
        writeln!(cursor)
    })?;

    cursor.flush()?;

    Ok((cursor.into_inner(), structs))
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
