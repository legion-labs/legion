use std::{path::PathBuf, str::FromStr};

use clap::{AppSettings, Arg, SubCommand};
use legion_asset_store::compiled_asset_store::CompiledAssetStoreAddr;
use legion_data_build::{DataBuildOptions, ResourcePath};
use legion_data_compiler::{Locale, Platform, Target};

const ARG_NAME_RESOURCE: &str = "resource";
const ARG_NAME_BUILDINDEX: &str = "buildindex";
const ARG_NAME_CAS: &str = "cas";
const ARG_NAME_MANIFEST: &str = "manifest";
const ARG_NAME_TARGET: &str = "target";
const ARG_NAME_PLATFORM: &str = "platform";
const ARG_NAME_LOCALE: &str = "locale";

fn main() -> Result<(), String> {
    let matches = clap::App::new("Data Build")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("Data Build CLI")
        .subcommand(
            SubCommand::with_name("compile")
                .about("Compile input resource.")
                .arg(
                    Arg::with_name(ARG_NAME_RESOURCE)
                        .required(true)
                        .help("Source to compile"),
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
                        .default_value("output.manifest")
                        .long(ARG_NAME_MANIFEST)
                        .help("Manifest file path."),
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

    if let ("compile", Some(cmd_args)) = matches.subcommand() {
        let source = cmd_args.value_of(ARG_NAME_RESOURCE).unwrap();
        let manifest = cmd_args.value_of(ARG_NAME_MANIFEST).unwrap();
        let target = cmd_args.value_of(ARG_NAME_TARGET).unwrap();
        let platform = cmd_args.value_of(ARG_NAME_PLATFORM).unwrap();
        let locale = cmd_args.value_of(ARG_NAME_LOCALE).unwrap();
        let source_name = ResourcePath::from_str(source).map_err(|_e| "Invalid ResourcePath")?;
        let manifest_file = PathBuf::from_str(manifest).map_err(|_e| "Invalid Manifest name")?;
        let target = Target::from_str(target).map_err(|_e| "Invalid Target")?;
        let platform = Platform::from_str(platform).map_err(|_e| "Invalid Platform")?;
        let locale = Locale::new(locale);
        let asset_store_path =
            CompiledAssetStoreAddr::from(cmd_args.value_of(ARG_NAME_CAS).unwrap());
        let buildindex_path = PathBuf::from(cmd_args.value_of(ARG_NAME_BUILDINDEX).unwrap());

        let mut config = DataBuildOptions::new(buildindex_path);
        config.asset_store(&asset_store_path);
        if let Ok(cwd) = std::env::current_dir() {
            config.compiler_dir(cwd);
        }
        if let Some(mut exe_dir) = std::env::args().next().map(|s| PathBuf::from(&s)) {
            if exe_dir.pop() && exe_dir.is_dir() {
                config.compiler_dir(exe_dir);
            }
        }

        let mut build = config.open().map_err(|_e| "Failed to open build index")?;
        let output = build
            .compile_named(&source_name, &manifest_file, target, platform, &locale)
            .map_err(|e| format!("Compilation Failed: '{}'", e))?;

        println!("{:?}", output);
    }
    Ok(())
}
