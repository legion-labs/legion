use clap::{App, Arg, SubCommand};
use std::fs;
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
    Ok(())
}

fn main() {
    let matches = App::new("Legion Source Control")
        .version("0.1.0")
        .subcommand(
            SubCommand::with_name("init-local")
                .about("Initializes a repository stored on a local filesystem")
                .arg(
                    Arg::with_name("directory")
                        .short("d")
                        .value_name("directory")
                        .required(true)
                        .help("lsc database directory"),
                ),
        )
        .get_matches();

    if let Some(init_local_match) = matches.subcommand_matches("init-local") {
        if let Err(e) = init_local_repository(init_local_match.value_of("directory").unwrap()) {
            println!("init_local_repository failed: {}", e);
            std::process::exit(1);
        }
    }
}
