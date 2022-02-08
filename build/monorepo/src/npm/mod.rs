use clap::Subcommand;
use lgn_tracing::span_fn;

use crate::{context::Context, Result};

pub mod build;
pub mod check;
pub mod clean;
pub mod fmt;
pub mod install;
pub mod lint;
pub mod test;
pub mod utils;

#[derive(Subcommand)]
pub enum NpmCommands {
    /// Install all npm packages recursively
    Install,

    /// Build an npm package that exposes a "build" script
    /// Recursively build all packages unless a package name is provided
    #[clap(name = "build")]
    Build(build::Args),

    /// Check an npm package that exposes a "check" script
    /// Recursively check all packages unless a package name is provided
    #[clap(name = "check")]
    Check(check::Args),

    /// Clean an npm package that exposes a "clean" script
    /// Recursively clean all packages unless a package name is provided
    #[clap(name = "clean")]
    Clean(clean::Args),

    /// Format an npm package that exposes an "fmt" script
    /// Recursively format all packages unless a package name is provided
    #[clap(name = "fmt")]
    Fmt(fmt::Args),

    /// Lint an npm package that exposes a "lint" script
    /// Recursively lint all packages unless a package name is provided
    #[clap(name = "lint")]
    Lint(lint::Args),

    /// Test an npm package that exposes a "test" script
    /// Recursively test all packages unless a package name is provided
    #[clap(name = "test")]
    Test(test::Args),
}

#[span_fn]
pub fn run(cmd: NpmCommands, ctx: &Context) -> Result<()> {
    match cmd {
        NpmCommands::Install => install::run(ctx),
        NpmCommands::Build(args) => build::run(&args, ctx),
        NpmCommands::Check(args) => check::run(&args, ctx),
        NpmCommands::Clean(args) => clean::run(&args, ctx),
        NpmCommands::Fmt(args) => fmt::run(&args, ctx),
        NpmCommands::Lint(args) => lint::run(&args, ctx),
        NpmCommands::Test(args) => test::run(&args, ctx),
    }
}
