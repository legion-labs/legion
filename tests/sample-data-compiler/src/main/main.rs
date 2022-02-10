//! The runtime server is the portion of the Legion Engine that runs off runtime
//! data to simulate a world. It is tied to the lifetime of a runtime client.
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)

// crate-specific lint exceptions:
//#![allow()]

use std::{fs, path::PathBuf};

use clap::{AppSettings, Parser};
use lgn_data_offline::resource::ResourcePathName;
use sample_data_compiler::{offline_compiler, raw_loader};

#[derive(Parser, Default)]
#[clap(name = "Sample data compiler")]
#[clap(
    about = "Will load RON files containing sample data, and generate offline resources and runtime assets, along with manifests.",
    version,
    author
)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
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
    let args = Args::parse();

    if args.clean {
        clean_folders(&args.root);
    }

    // generate contents of offline folder, from raw RON content
    raw_loader::build_offline(&args.root).await;

    // compile offline resources to runtime assets
    offline_compiler::build(&args.root, &ResourcePathName::from(&args.resource)).await;
}

fn clean_folders(project_dir: &str) {
    let mut can_clean = true;
    let mut path = PathBuf::from(project_dir);

    let mut test = |sub_path| {
        path.push(sub_path);
        can_clean &= path.exists();
        path.pop();
    };
    test("remote");
    test("offline");
    test("runtime");
    test("temp");

    if !can_clean {
        println!("Cannot clean folders in path {}", project_dir);
    } else {
        let mut delete = |sub_path, as_dir| {
            path.push(sub_path);
            let remove = if as_dir {
                fs::remove_dir_all
            } else {
                fs::remove_file
            };
            remove(path.as_path()).unwrap_or_else(|_| panic!("Cannot delete {:?}", path));
            path.pop();
        };
        delete("remote", true);
        delete("offline", true);
        delete("runtime", true);
        delete("temp", true);
    }
}
