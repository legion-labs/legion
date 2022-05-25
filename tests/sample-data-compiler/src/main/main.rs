//! The runtime server is the portion of the Legion Engine that runs off runtime
//! data to simulate a world. It is tied to the lifetime of a runtime client.
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)

// crate-specific lint exceptions:
//#![allow()]

use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use clap::Parser;
use lgn_data_offline::resource::ResourcePathName;
use lgn_source_control::RepositoryName;
use sample_data_compiler::{offline_compiler, raw_loader};

fn target_dir() -> PathBuf {
    std::env::current_exe().ok().map_or_else(
        || panic!("cannot find test directory"),
        |mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        },
    )
}

pub fn workspace_dir() -> PathBuf {
    target_dir()
        .as_path()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

#[derive(Parser, Default)]
#[clap(name = "Sample data compiler")]
#[clap(
    about = "Will load RON files containing sample data, and generate offline resources and runtime assets, along with manifests.",
    version,
    author
)]
#[clap(arg_required_else_help(false))]
struct Args {
    /// Folder containing raw/ directory
    #[clap(long, default_value = "tests/sample-data")]
    root: String,
    /// Path name of the resource to compile
    #[clap(long, default_value = "/world/sample_1.ent")]
    resource: String,
    /// Clean old folders from the target folder
    #[clap(short, long)]
    clean: bool,
}

#[tokio::main]
async fn main() {
    let _telemetry_guard = lgn_telemetry_sink::TelemetryGuard::default().unwrap();

    let args = Args::parse();

    if args.clean {
        clean_folders(&args.root);
    }

    let absolute_root = {
        let root_path = PathBuf::from(args.root);
        if root_path.is_absolute() {
            root_path
        } else {
            std::env::current_dir().unwrap().join(root_path)
        }
    };

    let repository_index = lgn_source_control::Config::load_and_instantiate_repository_index()
        .await
        .unwrap();
    let repository_name: RepositoryName = "sample-data".parse().unwrap();
    let branch_name = "main";

    // Ensure the repository exists.
    let _index = repository_index.ensure_repository(&repository_name).await;

    let source_control_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_persistent_provider()
            .await
            .unwrap(),
    );
    let data_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_volatile_provider()
            .await
            .unwrap(),
    );

    // generate contents of offline folder, from raw RON content
    let project = raw_loader::build_offline(
        &absolute_root,
        &repository_index,
        &repository_name,
        branch_name,
        source_control_content_provider,
        true,
    )
    .await;

    // compile offline resources to runtime assets
    offline_compiler::build(
        &project,
        &absolute_root,
        &ResourcePathName::from(&args.resource),
        Arc::clone(&data_content_provider),
    )
    .await;
}

fn clean_folders(project_dir: &str) {
    let delete = |sub_path: &str, as_dir| {
        let mut path = if Path::new(project_dir).is_relative() {
            env::current_dir().unwrap().join(project_dir)
        } else {
            PathBuf::from(project_dir)
        };
        path.push(sub_path);
        if !path.exists() {
            return;
        }
        let remove = if as_dir {
            fs::remove_dir_all
        } else {
            fs::remove_file
        };
        remove(&path).unwrap_or_default();
    };

    let builddb_dir = workspace_dir().join("target").join("build_db");

    delete("VERSION", false);
    delete("offline", true);
    delete("runtime", true);
    delete("temp", true);
    delete(builddb_dir.as_os_str().to_str().unwrap(), true);
}
