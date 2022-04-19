// crate-specific lint exceptions:
//#![allow()]

use std::{path::PathBuf, sync::Arc};

use clap::{Parser, Subcommand};
use lgn_data_build::{DataBuild, DataBuildOptions};
use lgn_data_compiler::{
    compiler_api::CompilationEnv, compiler_node::CompilerRegistryOptions, Locale,
};
use lgn_data_offline::Project;
use lgn_data_runtime::{ResourcePathId, ResourceType, ResourceTypeAndId};

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
        /// Path to build output database.
        #[clap(long = "output")]
        build_output: String,
        /// Name of the source control repository.
        #[clap(long)]
        repository_name: String,
        /// Name of the source control branch.
        #[clap(long)]
        branch_name: String,
    },
    /// Compile input resource
    #[clap(name = "compile")]
    Compile {
        /// Path in build graph to compile.
        resource: String,
        /// Name of the source control repository.
        #[clap(long)]
        repository_name: String,
        /// Name of the source control branch.
        #[clap(long)]
        branch_name: String,
        /// Build index file.
        #[clap(long = "output")]
        build_output: String,
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

    let cwd = std::env::current_dir().unwrap();
    let repository_index = lgn_source_control::Config::load_and_instantiate_repository_index()
        .await
        .map_err(|e| format!("failed creating repository index {}", e))?;
    let source_control_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_persistent_provider()
            .await
            .map_err(|err| format!("{:?}", err))?,
    );
    let data_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_volatile_provider()
            .await
            .map_err(|err| format!("{:?}", err))?,
    );

    match args.command {
        Commands::Create {
            build_output,
            repository_name,
            branch_name,
        } => {
            let repository_name = repository_name
                .parse()
                .map_err(|_e| format!("Invalid repository name '{}'", repository_name))?;

            let project = Project::new(
                repository_index,
                &repository_name,
                &branch_name,
                Arc::clone(&source_control_content_provider),
            )
            .await
            .map_err(|e| format!("failed to open project {}", e))?;

            let mut build = DataBuildOptions::new(
                DataBuildOptions::output_db_path(&build_output, &cwd, DataBuild::version()),
                Arc::clone(&source_control_content_provider),
                Arc::clone(&data_content_provider),
                CompilerRegistryOptions::default(),
            )
            .create(&project)
            .await
            .map_err(|e| format!("failed creating build index {}", e))?;

            if let Err(e) = build.source_pull(&project).await {
                eprintln!("Source Pull failed with '{}'", e);
            }
        }
        Commands::Compile {
            resource,
            repository_name,
            branch_name,
            build_output,
            runtime_flag,
            target,
            platform,
            locale,
        } => {
            let repository_name = repository_name
                .parse()
                .map_err(|_e| format!("Invalid repository name '{}'", repository_name))?;
            let target = target
                .parse()
                .map_err(|_e| format!("Invalid Target '{}'", target))?;
            let platform = platform
                .parse()
                .map_err(|_e| format!("Invalid Platform '{}'", platform))?;
            let locale = Locale::new(&locale);

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

            let project = Project::new(
                repository_index,
                &repository_name,
                &branch_name,
                Arc::clone(&source_control_content_provider),
            )
            .await
            .map_err(|e| e.to_string())?;

            let mut build = DataBuildOptions::new(
                DataBuildOptions::output_db_path(&build_output, &cwd, DataBuild::version()),
                Arc::clone(&source_control_content_provider),
                Arc::clone(&data_content_provider),
                compilers,
            )
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
                #[allow(clippy::same_functions_in_if_condition)]
                if let Ok(id) = resource.parse::<ResourceTypeAndId>() {
                    build
                        .lookup_pathid(id)
                        .await
                        .map_err(|e| e.to_string())?
                        .ok_or(format!(
                            "Cannot resolve ResourceId to ResourcePathId: '{}'",
                            resource
                        ))?
                } else if let Ok(id) = resource.parse() {
                    id
                } else if let Ok(name) = resource.parse() {
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
                let manifest_id = output
                    .into_rt_manifest(&data_content_provider, |_| true)
                    .await;
                println!("{}", manifest_id);
            } else {
                let output = serde_json::to_string(&output).unwrap();
                println!("{}", output);
            }
        }
    }
    Ok(())
}
