mod lsc_lib;

use lsc_lib::*;

use clap::{App, AppSettings, Arg, SubCommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::result::Result;

#[derive(Serialize, Deserialize, Debug)]
struct LocalChange {
    relative_path: String,
    change_type: String, //edit, add, delete
}

fn find_workspace_root(directory: &Path) -> Result<&Path, String> {
    if let Ok(_meta) = fs::metadata(directory.join(".lsc")) {
        return Ok(directory);
    }
    match directory.parent() {
        None => Err(String::from("workspace not found")),
        Some(parent) => find_workspace_root(parent),
    }
}

fn make_path_absolute(p: &Path) -> PathBuf {
    //fs::canonicalize is a trap - it generates crazy unusable "extended length" paths
    if p.is_absolute() {
        PathBuf::from(path_clean::clean(p.to_str().unwrap()))
    } else {
        PathBuf::from(path_clean::clean(
            std::env::current_dir().unwrap().join(p).to_str().unwrap(),
        ))
    }
}

fn path_relative_to(p: &Path, base: &Path) -> Result<PathBuf, String> {
    match p.strip_prefix(base) {
        Ok(res) => Ok(res.to_path_buf()),
        Err(e) => Err(format!("{:?} not relative to {:?}: {}", p, base, e)),
    }
}

fn track_new_file(file_to_add_specified: &Path) -> Result<(), String> {
    let file_to_add_buf = make_path_absolute(file_to_add_specified);
    let file_to_add = file_to_add_buf.as_path();
    match fs::metadata(file_to_add) {
        Ok(_file_metadata) => {
            match file_to_add.parent() {
                None => {
                    return Err(format!(
                        "Error finding parent workspace of {:?}",
                        file_to_add
                    ));
                }
                Some(parent) => {
                    let workspace_root = make_path_absolute(find_workspace_root(parent)?);
                    let local_edit_id = uuid::Uuid::new_v4().to_string();
                    let local_edit_obj_path = workspace_root
                        .join(".lsc/local_edits/")
                        .join(local_edit_id + ".json");

                    //todo: lock the new file before recording the local change
                    let local_change = LocalChange {
                        relative_path: path_to_string(
                            path_relative_to(file_to_add, workspace_root.as_path())?.as_path(),
                        ),
                        change_type: String::from("add"),
                    };

                    match serde_json::to_string(&local_change) {
                        Ok(json_spec) => {
                            write_file(local_edit_obj_path.as_path(), json_spec.as_bytes())?;
                        }
                        Err(e) => {
                            return Err(format!("Error formatting local change spec: {}", e));
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Error reading file metadata {:?}: {}",
                file_to_add, e
            ))
        }
    }
    Ok(())
}

fn main() {
    let matches = App::new("Legion Source Control")
        .version("0.1.0")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("init-local-repository")
                .about("Initializes a repository stored on a local filesystem")
                .arg(
                    Arg::with_name("repository-directory")
                        .short("r")
                        .value_name("repository-directory")
                        .required(true)
                        .help("lsc database directory"),
                ),
        )
        .subcommand(
            SubCommand::with_name("init-workspace")
                .about("Initializes a workspace and populates it with the latest version of the main branch")
                .arg(
                    Arg::with_name("workspace-directory")
                        .short("w")
                        .value_name("workspace-directory")
                        .required(true)
                        .help("lsc workspace directory"))
                .arg(
                    Arg::with_name("repository-directory")
                        .short("r")
                        .value_name("repository-directory")
                        .required(true)
                        .help("local repository directory"),
                ),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds local file to the set of pending changes")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("local path within a workspace")),
        )
        .get_matches();

    match matches.subcommand() {
        ("init-local-repository", Some(command_match)) => {
            if let Err(e) =
                lsc_lib::init_local_repository(command_match.value_of("repository-directory").unwrap())
            {
                println!("init_local_repository failed: {}", e);
                std::process::exit(1);
            }
        }
        ("init-workspace", Some(command_match)) => {
            if let Err(e) = init_workspace(
                Path::new(command_match.value_of("workspace-directory").unwrap()),
                Path::new(command_match.value_of("repository-directory").unwrap()),
            ) {
                println!("init_workspace failed: {}", e);
                std::process::exit(1);
            }
        }
        ("add", Some(command_match)) => {
            if let Err(e) = track_new_file(Path::new(command_match.value_of("path").unwrap())) {
                println!("add failed: {}", e);
                std::process::exit(1);
            }
        }
        _ => {}
    }
}
