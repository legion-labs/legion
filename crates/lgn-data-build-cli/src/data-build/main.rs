// crate-specific lint exceptions:
//#![allow()]

use std::{path::PathBuf, str::FromStr};

use clap::{AppSettings, Parser, Subcommand};
use lgn_content_store::ContentStoreAddr;
use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::{
    compiler_api::CompilationEnv, compiler_node::CompilerRegistryOptions, Locale, Platform, Target,
};
use lgn_data_offline::ResourcePathId;
use lgn_data_runtime::ResourceTypeAndId;

#[derive(Parser, Debug)]
#[clap(name = "Data Build")]
#[clap(about = "Data Build CLI", version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create build index at a specified location
    #[clap(name = "create")]
    Create {
        /// New build index path.
        build_index: PathBuf,
        /// Source project path.
        #[clap(long)]
        project: PathBuf,
    },
    /// Compile input resource
    #[clap(name = "compile")]
    Compile {
        /// Path in build graph to compile.
        resource: String,
        /// BBuild index file.
        #[clap(long = "buildindex")]
        build_index: PathBuf,
        /// Compiled Asset Store addresses where assets will be output.
        #[clap(long)]
        cas: String,
        /// Manifest file path.
        #[clap(long)]
        manifest: Option<PathBuf>,
        /// Accept ResourceId as the compilation input and output a runtime manifest.
        #[clap(long = "rt")]
        runtime_flag: bool,
        /// Build target (Game, Server, etc).
        #[clap(long)]
        target: String,
        /// Build platform (Windows, Unix, etc).
        #[clap(long)]
        platform: String,
        /// Build localization (en, fr, etc).
        #[clap(long)]
        locale: String,
    },
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Cli::parse();

    match args.command {
        Commands::Create {
            build_index,
            project,
        } => {
            let mut build = DataBuildOptions::new(&build_index, CompilerRegistryOptions::default())
                .content_store(&ContentStoreAddr::from("."))
                .create(project)
                .await
                .map_err(|e| format!("failed creating build index {}", e))?;

            if let Err(e) = build.source_pull() {
                eprintln!("Source Pull failed with '{}'", e);
                let _res = std::fs::remove_file(build_index);
            }
        }
        Commands::Compile {
            resource,
            build_index,
            cas,
            manifest,
            runtime_flag,
            target,
            platform,
            locale,
        } => {
            let target =
                Target::from_str(&target).map_err(|_e| format!("Invalid Target '{}'", target))?;
            let platform = Platform::from_str(&platform)
                .map_err(|_e| format!("Invalid Platform '{}'", platform))?;
            let locale = Locale::new(&locale);
            let content_store_path = ContentStoreAddr::from(cas.as_str());

            let compilers = std::env::args()
                .next()
                .and_then(|s| {
                    let mut exe_dir = PathBuf::from(&s);
                    if exe_dir.pop() && exe_dir.is_dir() {
                        Some(CompilerRegistryOptions::from_dir(&exe_dir))
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let mut build = DataBuildOptions::new(build_index, compilers)
                .content_store(&content_store_path)
                .open()
                .await
                .map_err(|e| format!("Failed to open build index: '{}'", e))?;

            let derived = {
                if runtime_flag {
                    let id = resource
                        .parse::<ResourceTypeAndId>()
                        .map_err(|_e| format!("Invalid Resource (ResourceId) '{}'", resource))?;
                    build.lookup_pathid(id).ok_or(format!(
                        "Cannot resolve ResourceId to ResourcePathId: '{}'",
                        id
                    ))?
                } else {
                    ResourcePathId::from_str(&resource)
                        .map_err(|_e| format!("Invalid Resource (ResourcePathId) '{}'", resource))?
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
                .compile(
                    derived,
                    manifest,
                    &CompilationEnv {
                        target,
                        platform,
                        locale,
                    },
                )
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
    }
    Ok(())
}
