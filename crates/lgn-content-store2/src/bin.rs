use std::path::PathBuf;

use clap::{Parser, Subcommand};
use lgn_content_store2::Identifier;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Get { identifier: Identifier },
    Put { file_path: PathBuf },
}

fn main() {
    let args: Args = Args::parse();

    match args.command {
        Commands::Get { identifier } => {
            println!("{}", identifier);
        }
        Commands::Put { file_path } => {
            println!("{}", file_path.display());
        }
    }
}
