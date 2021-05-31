use clap::{App, Arg, SubCommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::prelude::*;
use std::result::Result;

fn init_local_repository(directory: &str) -> Result<(), String> {
    if let Ok(_) = fs::metadata(directory) {
        return Err(format!("{} already exists", directory));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/trees", directory)) {
        return Err(format!("Error creating trees directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/commits", directory)) {
        return Err(format!("Error creating commits directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/blobs", directory)) {
        return Err(format!("Error creating blobs directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/workspaces", directory)) {
        return Err(format!("Error creating workspaces directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/branches", directory)) {
        return Err(format!("Error creating branches directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/locks", directory)) {
        return Err(format!("Error creating locks directory: {}", e));
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Workspace {
    id: String,
    repository: String,
    owner: String,
}

fn init_workspace(workspace_directory: &str, repository_directory: &str) -> Result<(), String> {
    if let Ok(_) = fs::metadata(workspace_directory) {
        return Err(format!("{} already exists", workspace_directory));
    }
    if let Err(e) = fs::create_dir_all(format!("{}/.lsc", workspace_directory)) {
        return Err(format!("Error creating .lsc directory: {}", e));
    }
    let spec = Workspace {
        id: uuid::Uuid::new_v4().to_string(),
        repository: repository_directory.to_string(),
        owner: whoami::username(),
    };
    match serde_json::to_string(&spec) {
        Ok(json_spec) => {
            match fs::File::create(format!("{}/.lsc/workspace.json", workspace_directory)) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(json_spec.as_bytes()) {
                        return Err(format!("Error writing workspace.json: {}", e));
                    }
                }
                Err(e) => return Err(format!("Error writing workspace.json: {}", e)),
            }
        }
        Err(e) => {
            return Err(format!("Error formatting workspace spec: {}", e));
        }
    }
    Ok(())
}

fn main() {
    let matches = App::new("Legion Source Control")
        .version("0.1.0")
        .subcommand(
            SubCommand::with_name("init-local-repository")
                .about("Initializes a repository stored on a local filesystem.")
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
                .about("Initializes a workspace and populates it with the latest version of the main branch.")
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
        .get_matches();

    if let Some(command_match) = matches.subcommand_matches("init-local-repository") {
        if let Err(e) =
            init_local_repository(command_match.value_of("repository-directory").unwrap())
        {
            println!("init_local_repository failed: {}", e);
            std::process::exit(1);
        }
    }

    //todo: process in the order specified
    if let Some(command_match) = matches.subcommand_matches("init-workspace") {
        if let Err(e) = init_workspace(
            command_match.value_of("workspace-directory").unwrap(),
            command_match.value_of("repository-directory").unwrap(),
        ) {
            println!("init_workspace failed: {}", e);
            std::process::exit(1);
        }
    }
}
