//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use clap::{Parser, Subcommand};
use futures::Future;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lgn_content_store2::{
    Config, ContentReader, ContentWriterExt, Identifier, MonitorProvider, TransferCallbacks,
};
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

        file_path: PathBuf,
    },
    Put {
        file_path: Option<PathBuf>,
    },
}

struct TransferProgress {
    progress: Arc<MultiProgress>,
    exists_progress_style: ProgressStyle,
    progress_style: ProgressStyle,
    bars: Arc<RwLock<HashMap<Identifier, ProgressBar>>>,

    #[allow(dead_code)]
    hidden_bar: Arc<ProgressBar>,
}

impl TransferProgress {
    fn new() -> Self {
        let progress = Arc::new(MultiProgress::new());
        let exists_progress_style = ProgressStyle::default_bar().template(
            "{prefix} [{elapsed_precise}] {wide_bar:.green/darkgreen} {bytes}/{total_bytes} {msg}",
        );
        let progress_style = ProgressStyle::default_bar()
                .template("{prefix} [{elapsed_precise}] {wide_bar:.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}");
        let bars = Arc::new(RwLock::new(HashMap::new()));

        // Let's add a hidden bar to the progress bar that the progress stays
        // alive until we explicitely shut it down.
        let hidden_bar = Arc::new(progress.add(ProgressBar::hidden()));

        Self {
            progress,
            exists_progress_style,
            progress_style,
            bars,
            hidden_bar,
        }
    }

    fn join(&self) -> impl Future<Output = anyhow::Result<()>> {
        let progress = Arc::clone(&self.progress);

        async move {
            tokio::task::spawn_blocking(move || progress.join())
                .await?
                .map_err(|err| anyhow::anyhow!("failed to join the progress bar thread: {}", err))
        }
    }
}

impl TransferCallbacks for TransferProgress {
    fn on_transfer_avoided(&self, id: &Identifier) {
        let bar = self
            .progress
            .add(ProgressBar::new(id.data_size().try_into().unwrap()));
        bar.set_style(self.exists_progress_style.clone());
        bar.set_prefix(id.to_string());
        bar.set_position(id.data_size().try_into().unwrap());
        bar.finish_with_message("Exists");

        self.bars.write().unwrap().insert(id.clone(), bar);
    }

    fn on_transfer_started(&self, id: &Identifier) {
        let bar = self
            .progress
            .add(ProgressBar::new(id.data_size().try_into().unwrap()));
        bar.set_style(self.progress_style.clone());
        bar.set_prefix(id.to_string());

        self.bars.write().unwrap().insert(id.clone(), bar);
    }

    fn on_transfer_progress(&self, id: &Identifier, increment: usize, _total: usize) {
        if let Some(bar) = self.bars.read().unwrap().get(id) {
            bar.inc(increment.try_into().unwrap());
        }
    }

    fn on_transfer_stopped(&self, id: &Identifier, result: lgn_content_store2::Result<usize>) {
        if let Some(bar) = self.bars.read().unwrap().get(id) {
            match result {
                Ok(_) => bar.finish_with_message("Done"),
                Err(err) => bar.abandon_with_message(format!("{}", err)),
            }
        }
    }
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
    let provider = MonitorProvider::new(provider);

    let transfer_progress = TransferProgress::new();
    let transfer_join = transfer_progress.join();

    match args.command {
        Commands::Get {
            identifier,
            file_path,
        } => {
            let provider = provider.on_download_callbacks(transfer_progress);

            let mut output =
                Box::new(tokio::fs::File::create(file_path).await.map_err(|err| {
                    anyhow::anyhow!("failed to create destination file: {}", err)
                })?);

            let copy = async move {
                let mut input = provider
                    .get_content_reader(&identifier)
                    .await
                    .map_err(|err| anyhow::anyhow!("failed to get asset: {}", err))?;

                tokio::io::copy_buf(
                    &mut tokio::io::BufReader::with_capacity(10 * 1024 * 1024, &mut input),
                    &mut output,
                )
                .await
                .map_err(|err| anyhow::anyhow!("failed to copy asset: {}", err))
                .map(|_| ())
            };

            let res = futures::join!(copy, transfer_join);

            res.0?;
            res.1?;
        }
        Commands::Put { file_path } => {
            let provider = provider.on_upload_callbacks(transfer_progress);

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

            let copy = async move {
                provider
                    .write_content(&buf)
                    .await
                    .map_err(|err| anyhow::anyhow!("failed to write asset: {}", err))
            };

            let res = futures::join!(copy, transfer_join);

            let id = res.0?;
            res.1?;

            println!("{}", id);
        }
    }

    Ok(())
}
