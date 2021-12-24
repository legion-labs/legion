use crate::context::Context;
use crate::Result;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(long, short)]
    pub(crate) git_ref: Option<String>,
}

pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let files = ctx.get_changed_files(args.git_ref.as_deref().unwrap_or("HEAD"))?;
    for file in files {
        println!("{}", file.display());
    }
    Ok(())
}
