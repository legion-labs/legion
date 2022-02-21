use lgn_tracing::span_fn;

use crate::{cargo::Cargo, context::Context, Error, Result};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    #[clap(name = "test")]
    /// Run tests and then reviews
    Test,
    #[clap(name = "review")]
    /// Interactively review snapshots
    Review,
    #[clap(name = "reject")]
    /// Rejects all snapshots
    Reject,
    #[clap(name = "accept")]
    /// Accept all snapshots
    Accept,
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let mut cmd = Cargo::new(ctx, "insta", true);

    let args = match args.command {
        Commands::Test => ["test"],
        Commands::Review => ["review"],
        Commands::Reject => ["reject"],
        Commands::Accept => ["accept"],
    };

    if cmd.args(args).run().is_ok() {
        Ok(())
    } else {
        if !ctx.installer().install_via_cargo_if_needed("cargo-insta") {
            return Err(Error::new("could not find/install cargo-insta"));
        }
        cmd.run()
    }
}
