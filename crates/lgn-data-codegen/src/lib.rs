//! Code generation module for Data Model

// crate-specific lint exceptions:
#![warn(missing_docs)]

mod compiler_codegen;
mod component_codegen;
mod enum_codegen;
mod resource_codegen;
mod runtime_codegen;
mod struct_codegen;

mod attributes;
mod enum_meta_info;
mod member_meta_info;
mod struct_meta_info;

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{error::Error, io::Cursor};

use enum_meta_info::EnumMetaInfo;
use lgn_build_utils::symlink_out_dir;
use quote::{format_ident, ToTokens};
use struct_meta_info::StructMetaInfo;
use syn::ItemUse;

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
pub fn generate_def() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(&std::env::var("OUT_DIR").unwrap());
    let directory = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("def");
    println!("cargo:rerun-if-changed={}", directory.display());
    generate_for_directory(&directory, out_dir)
        .and_then(|_| symlink_out_dir().map_err(std::convert::Into::into))
}

fn generate_for_directory(
    src_dir: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let src_dir = src_dir.as_ref();
    let out_dir = out_dir.as_ref();
    std::fs::create_dir_all(out_dir)?;
    let codegen_file_path = out_dir.join("data_def.rs");
    let mut codegen_file = std::fs::File::create(&codegen_file_path)?;

    // Extract all the MetaInfo from the rust type
    let mut processed_sub_mods = HashMap::<String, ModuleMetaInfo>::new();
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
                let sub_mod_name = filename.to_str().unwrap().strip_suffix(".rs").unwrap();
                processed_sub_mods.insert(sub_mod_name.into(), extract_meta_infos(&path));
            }
        }
    }

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

            for (sub_mod_name, info) in &processed_sub_mods {
                let content =
                    generate_data_definition(&info.struct_meta_infos, &info.uses, gen_type)?;
                writeln!(codegen_file, "mod {} {{", sub_mod_name)?;
                codegen_file.write_all(&content)?;
                writeln!(codegen_file, "}}")?;
                writeln!(codegen_file, "pub use {}::*;\n", sub_mod_name)?;
            }

            // Add Registration/Loader code
            let out_token = if gen_type == GenerationType::OfflineFormat {
                resource_codegen::generate_registration_code(&processed_sub_mods)
            } else {
                runtime_codegen::generate_registration_code(&processed_sub_mods)
            };
            codegen_file.write_all(out_token.to_string().as_bytes())?;

            // End of the gentype module
            writeln!(codegen_file, "}}")?;
            Ok(())
        })?;

    // Write Enums top module
    codegen_file.write_all(
        enum_codegen::generate_reflection(&processed_sub_mods)
            .to_string()
            .as_bytes(),
    )?;

    codegen_file.flush()?;
    Command::new("rustfmt")
        .args(&[codegen_file_path.as_os_str()])
        .status()?;

    Ok(())
}

struct ModuleMetaInfo {
    struct_meta_infos: Vec<StructMetaInfo>,
    enum_meta_infos: Vec<EnumMetaInfo>,
    uses: Vec<ItemUse>,
}

fn extract_meta_infos(source_path: &std::path::Path) -> ModuleMetaInfo {
    let src = std::fs::read_to_string(source_path).expect("Read file");
    let ast = syn::parse_file(&src).expect("Unable to parse file");

    // Gather info about the structs
    let struct_meta_infos: Vec<StructMetaInfo> = ast
        .items
        .iter()
        .filter_map(|item| match &item {
            syn::Item::Struct(item_struct) => Some(StructMetaInfo::new(item_struct)),
            _ => None,
        })
        .collect();

    // Gather info about the enum
    let enum_meta_infos: Vec<EnumMetaInfo> = ast
        .items
        .iter()
        .filter_map(|item| match &item {
            syn::Item::Enum(item_enum) => Some(EnumMetaInfo::new(item_enum)),
            _ => None,
        })
        .collect();

    // Gather uses info
    let uses = ast
        .items
        .iter()
        .filter_map(|item| match &item {
            syn::Item::Use(uses) => Some(uses.clone()),
            _ => None,
        })
        .collect::<Vec<ItemUse>>();

    ModuleMetaInfo {
        struct_meta_infos,
        enum_meta_infos,
        uses,
    }
}

/// Default Code Generator (called from Build Scripts)
/// # Errors
fn generate_data_definition(
    structs: &[StructMetaInfo],
    uses: &[ItemUse],
    gen_type: GenerationType,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut cursor = Cursor::new(vec![]);

    // Write 'uses' from definition
    uses.iter()
        .try_for_each(|ts| cursor.write_all(ts.to_token_stream().to_string().as_bytes()))?;

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
        #[allow(clippy::wildcard_imports)]
        use crate::*;
        #(use #imports;)*
    };
    cursor.write_all(imports.to_string().as_bytes())?;

    // Generate struct code
    structs.iter().try_for_each(|meta_info| {
        let out_token = struct_codegen::generate_reflection(meta_info, gen_type);
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

    Ok(cursor.into_inner())
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
            let meta_info = crate::struct_meta_info::StructMetaInfo::new(item_struct);
            if meta_info.is_resource {
                let gen_path =
                    std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join(Path::new(
                        &format!("compiler_{}.rs", &meta_info.name.to_string().to_lowercase()),
                    ));

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
