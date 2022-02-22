use std::path::PathBuf;

use clap::{Parser, Subcommand};
use lgn_content_store2::{Config, ContentWriterExt, Identifier};
use tokio::io::AsyncReadExt;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Get {
        identifier: Identifier,

        file_path: Option<PathBuf>,
    },
    Put {
        file_path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();
    let config = Config::new();
    let provider = config
        .provider
        .new_provider()
        .await
        .map_err(|err| anyhow::anyhow!("failed to create content provider: {}", err))?;

    match args.command {
        Commands::Get {
            identifier,
            file_path,
        } => {
            let mut input = provider
                .get_content_reader(&identifier)
                .await
                .map_err(|err| anyhow::anyhow!("failed to get asset: {}", err))?;

            let mut output: Box<dyn tokio::io::AsyncWrite + Unpin> = match file_path {
                Some(file_path) => {
                    Box::new(tokio::fs::File::create(file_path).await.map_err(|err| {
                        anyhow::anyhow!("failed to create destination file: {}", err)
                    })?)
                }
                None => Box::new(tokio::io::stdout()),
            };

            tokio::io::copy(&mut input, &mut output)
                .await
                .map_err(|err| anyhow::anyhow!("failed to copy asset: {}", err))?;
        }
        Commands::Put { file_path } => {
            let mut input: Box<dyn tokio::io::AsyncRead + Unpin> = match file_path {
                Some(file_path) => {
                    Box::new(tokio::fs::File::open(file_path).await.map_err(|err| {
                        anyhow::anyhow!("failed to create destination file: {}", err)
                    })?)
                }
                None => Box::new(tokio::io::stdin()),
            };

            let mut buf = Vec::new();

            input
                .read_to_end(&mut buf)
                .await
                .map_err(|err| anyhow::anyhow!("failed to read input: {}", err))?;

            let id = provider
                .write_content(&buf)
                .await
                .map_err(|err| anyhow::anyhow!("failed to write asset: {}", err))?;

            println!("{}", id);
        }
    }

    Ok(())
}
