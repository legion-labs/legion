//! The runtime server is the portion of the Legion Engine that runs off runtime
//! data to simulate a world. It is tied to the lifetime of a runtime client.
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)

// crate-specific lint exceptions:
//#![allow()]

use std::{fs, path::PathBuf, sync::Arc};

use clap::Parser;
use lgn_data_offline::resource::ResourcePathName;
use lgn_source_control::RepositoryName;
use lgn_tracing::info;
use sample_data_compiler::{offline_compiler, raw_loader};

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
    let repository_name: RepositoryName = "tests-sample-data".parse().unwrap();

    // Always re-create the repository, even if it doesn't exist.
    let _index = repository_index
        .recreate_repository(repository_name.clone())
        .await;

    let source_control_content_provider = Arc::new(
        lgn_content_store2::Config::load_and_instantiate_persistent_provider()
            .await
            .unwrap(),
    );
    let data_content_provider = Arc::new(
        lgn_content_store2::Config::load_and_instantiate_volatile_provider()
            .await
            .unwrap(),
    );

    // generate contents of offline folder, from raw RON content
    raw_loader::build_offline(
        &absolute_root,
        &repository_index,
        repository_name,
        Arc::clone(&source_control_content_provider),
        true,
    )
    .await;

    // compile offline resources to runtime assets
    offline_compiler::build(
        &absolute_root,
        &ResourcePathName::from(&args.resource),
        repository_index,
        Arc::clone(&source_control_content_provider),
        Arc::clone(&data_content_provider),
    )
    .await;
}

fn clean_folders(project_dir: &str) {
    let mut can_clean = true;
    let path = PathBuf::from(project_dir);

    let mut test = |sub_path| {
        can_clean &= path.join(sub_path).exists();
    };
    test("offline");
    test("runtime");
    test("temp");

    if !can_clean {
        info!("Cannot clean folders in path {}", project_dir);
    } else {
        let delete = |sub_path, as_dir| {
            let remove = if as_dir {
                fs::remove_dir_all
            } else {
                fs::remove_file
            };
            remove(path.join(sub_path)).unwrap_or_else(|_| panic!("Cannot delete {:?}", path));
        };

        let _result = fs::remove_file(path.join("VERSION"));
        delete("offline", true);
        delete("runtime", true);
        delete("temp", true);
    }
}
