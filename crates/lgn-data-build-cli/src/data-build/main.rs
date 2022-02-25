// crate-specific lint exceptions:
//#![allow()]

use std::{path::PathBuf, str::FromStr};

use clap::{Parser, Subcommand};
use lgn_content_store::ContentStoreAddr;
use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::{
    compiler_api::CompilationEnv, compiler_node::CompilerRegistryOptions, Locale, Platform, Target,
};
use lgn_data_offline::{
    resource::{Project, ResourcePathName},
    ResourcePathId,
};
use lgn_data_runtime::{ResourceType, ResourceTypeAndId};

#[derive(Parser, Debug)]
#[clap(name = "Data Build")]
#[clap(about = "Data Build CLI", version, author)]
#[clap(arg_required_else_help(true))]
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
        /// Compiled Asset Store addresses where generated source index will be stored.
        #[clap(long)]
        cas: String,
    },
    /// Compile input resource
    #[clap(name = "compile")]
    Compile {
        /// Path in build graph to compile.
        resource: String,
        /// Source project path.
        #[clap(long = "project")]
        project: PathBuf,
        /// Build index file.
        #[clap(long = "buildindex")]
        build_index: PathBuf,
        /// Compiled Asset Store addresses where assets will be output.
        #[clap(long)]
        cas: String,
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
            cas,
        } => {
            let (mut build, project) =
                DataBuildOptions::new(&build_index, CompilerRegistryOptions::default())
                    .content_store(&ContentStoreAddr::from(&cas[..]))
                    .create_with_project(project)
                    .await
                    .map_err(|e| format!("failed creating build index {}", e))?;

            if let Err(e) = build.source_pull(&project).await {
                eprintln!("Source Pull failed with '{}'", e);
                let _res = std::fs::remove_file(build_index);
            }
        }
        Commands::Compile {
            resource,
            project,
            build_index,
            cas,
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
                        Some(CompilerRegistryOptions::local_compilers(&exe_dir))
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let project = Project::open(&project).await.map_err(|e| e.to_string())?;

            let mut build = DataBuildOptions::new(build_index, compilers)
                .content_store(&content_store_path)
                .open(&project)
                .await
                .map_err(|e| format!("Failed to open build index: '{}'", e))?;

            //
            // for now, each time we build we make sure we have a fresh input data indexed
            // by doing a source_pull. this should most likely be executed only on demand.
            //
            build
                .source_pull(&project)
                .await
                .map_err(|e| format!("Source Pull Failed: '{}'", e))?;

            let derived = {
                if let Ok(id) = resource.parse::<ResourceTypeAndId>() {
                    build
                        .lookup_pathid(id)
                        .await
                        .map_err(|e| e.to_string())?
                        .ok_or(format!(
                            "Cannot resolve ResourceId to ResourcePathId: '{}'",
                            resource
                        ))?
                } else if let Ok(id) = ResourcePathId::from_str(&resource) {
                    id
                } else if let Ok(name) = ResourcePathName::from_str(&resource) {
                    let id = project
                        .find_resource(&name)
                        .await
                        .map_err(|e| format!("Could not find source resource: '{}'", e))?;
                    ResourcePathId::from(id).push(ResourceType::new(b"runtime_entity"))
                } else {
                    return Err(format!("Could not parse resource input: '{}'", resource));
                }
            };

            let output = build
                .compile(
                    derived,
                    &CompilationEnv {
                        target,
                        platform,
                        locale,
                    },
                )
                .await
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
