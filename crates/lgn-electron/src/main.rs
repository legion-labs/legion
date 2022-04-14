//! Legion Electron

use clap::Parser;
use error::Result;
use types::{Args, Command};
use utils::ElectronPackageConfig;

mod build;
mod error;
mod remote;
mod start;
mod types;
mod utils;

fn main() -> Result<()> {
    let args = Args::parse();

    let command = args.command.clone();

    let electron_package_config = match args.package_path {
        None => ElectronPackageConfig::local(),
        Some(ref package_path) => ElectronPackageConfig::new(
            package_path,
            &args.main_path,
            &args.tsconfig_path,
            args.typescript,
        ),
    }?;

    let configuration = args.try_into()?;

    match command {
        Command::Remote { .. } => remote::run(&electron_package_config, &configuration),
        Command::Start { .. } => start::run(&electron_package_config, &configuration),
        Command::Build { .. } => build::run(&electron_package_config, &configuration),
    }?;

    Ok(())
}
