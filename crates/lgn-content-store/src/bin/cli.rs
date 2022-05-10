//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

use std::{
    cmp::min,
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use clap::{Parser, Subcommand};
use console::style;
use futures::Future;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lgn_content_store::{
    Config, Error, HashRef, Identifier, MonitorAsyncAdapter, TransferCallbacks,
};
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{async_span_scope, LevelFilter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,

    #[clap(short = 'd', long = "debug")]
    debug: bool,

    #[clap(long="section", short='s', default_value=Config::SECTION_PERSISTENT)]
    section: String,
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
    Explain {
        identifier: Identifier,
        #[clap(
            short = 's',
            long = "show-data",
            help = "Show the data of the identifier"
        )]
        show_data: bool,
    },
}

#[derive(Debug, Clone)]
struct TransferProgress {
    progress: Arc<MultiProgress>,
    progress_style: ProgressStyle,
    bars: Arc<RwLock<HashMap<String, ProgressBar>>>,

    #[allow(dead_code)]
    hidden_bar: Arc<ProgressBar>,
}

impl TransferProgress {
    fn new() -> Self {
        let progress = Arc::new(MultiProgress::new());
        let progress_style = ProgressStyle::default_bar()
                .template("{prefix:52!} [{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}");
        let bars = Arc::new(RwLock::new(HashMap::new()));

        // Let's add a hidden bar to the progress bar that the progress stays
        // alive until we explicitely shut it down.
        let hidden_bar = Arc::new(progress.add(ProgressBar::hidden()));

        Self {
            progress,
            progress_style,
            bars,
            hidden_bar,
        }
    }

    fn join(&self) -> impl Future<Output = anyhow::Result<()>> {
        let progress = Arc::clone(&self.progress);

        async move {
            tokio::task::spawn_blocking(move || progress.join_and_clear())
                .await?
                .map_err(|err| anyhow::anyhow!("failed to join the progress bar thread: {}", err))
        }
    }
}

impl TransferCallbacks<HashRef> for TransferProgress {
    fn on_transfer_avoided(&self, id: &HashRef, total: usize) {
        let bar = self
            .progress
            .add(ProgressBar::new(total.try_into().unwrap()));
        bar.set_style(self.progress_style.clone());
        bar.set_prefix(id.to_string());
        bar.set_position(id.data_size().try_into().unwrap());
        bar.finish_with_message("♥");

        self.bars.write().unwrap().insert(id.to_string(), bar);
    }

    fn on_transfer_started(&self, id: &HashRef, total: usize) {
        let bar = self
            .progress
            .add(ProgressBar::new(total.try_into().unwrap()));
        bar.set_style(self.progress_style.clone());
        bar.set_prefix(id.to_string());

        self.bars.write().unwrap().insert(id.to_string(), bar);
    }

    fn on_transfer_progress(&self, id: &HashRef, _total: usize, inc: usize, _current: usize) {
        if let Some(bar) = self.bars.read().unwrap().get(&id.to_string()) {
            bar.inc(inc.try_into().unwrap());
        }
    }

    fn on_transfer_stopped(
        &self,
        id: &HashRef,
        _total: usize,
        inc: usize,
        _current: usize,
        result: lgn_content_store::content_providers::Result<()>,
    ) {
        if let Some(bar) = self.bars.read().unwrap().get(&id.to_string()) {
            bar.inc(inc.try_into().unwrap());

            match result {
                Ok(_) => bar.finish_with_message("✔️"),
                Err(err) => bar.abandon_with_message(format!("{}", err)),
            }
        }
    }
}

impl TransferCallbacks<String> for TransferProgress {
    fn on_transfer_avoided(&self, _id: &String, _total: usize) {}

    fn on_transfer_started(&self, id: &String, total: usize) {
        let bar = self
            .progress
            .add(ProgressBar::new(total.try_into().unwrap()));
        bar.set_style(self.progress_style.clone());
        bar.set_prefix(id.clone());

        self.bars.write().unwrap().insert(id.clone(), bar);
    }

    fn on_transfer_progress(&self, id: &String, _total: usize, inc: usize, _current: usize) {
        if let Some(bar) = self.bars.read().unwrap().get(id) {
            bar.inc(inc.try_into().unwrap());
        }
    }

    fn on_transfer_stopped(
        &self,
        id: &String,
        _total: usize,
        inc: usize,
        _current: usize,
        result: lgn_content_store::content_providers::Result<()>,
    ) {
        if let Some(bar) = self.bars.read().unwrap().get(id) {
            bar.inc(inc.try_into().unwrap());

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

    let _telemetry_guard = TelemetryGuardBuilder::default()
        .with_local_sink_enabled(args.debug)
        .with_local_sink_max_level(LevelFilter::Debug)
        .build();

    async_span_scope!("lgn-content-store-srv::main");

    let config = Config::load(&args.section)?;
    let mut provider = config
        .instantiate_provider()
        .await
        .map_err(|err| anyhow::anyhow!("failed to create content provider: {}", err))?;

    // Let's add monitoring to the content-provider.
    let transfer_progress = TransferProgress::new();
    let file_transfer_progress = transfer_progress.clone();
    let transfer_join = transfer_progress.join();

    match args.command {
        Commands::Get {
            identifier,
            file_path,
        } => {
            provider.set_download_callbacks(transfer_progress);

            let output = Box::new(tokio::fs::File::create(&file_path).await.map_err(|err| {
                anyhow::anyhow!(
                    "failed to create destination file `{}`: {}",
                    file_path.display(),
                    err
                )
            })?);

            let mut input = provider
                .get_reader(&identifier)
                .await
                .map_err(|err| anyhow::anyhow!("failed to get content: {}", err))?;

            let mut output = MonitorAsyncAdapter::new(
                output,
                file_path.display().to_string(),
                input.size(),
                Arc::new(Box::new(file_transfer_progress)),
            );

            let copy = async move {
                tokio::io::copy_buf(
                    &mut tokio::io::BufReader::with_capacity(10 * 1024 * 1024, &mut input),
                    &mut output,
                )
                .await
                .map_err(|err| anyhow::anyhow!("failed to copy asset: {}", err))?;

                output
                    .shutdown()
                    .await
                    .map_err(|err| anyhow::anyhow!("failed to shutdown output: {}", err))
                    .map(|_| ())
            };

            let res = futures::join!(copy, transfer_join);

            res.0?;
            res.1?;
        }
        Commands::Put { file_path } => {
            provider.set_upload_callbacks(transfer_progress);

            let copy = async move {
                let buf = if let Some(file_path) = file_path {
                    let f = tokio::fs::File::open(&file_path).await.map_err(|err| {
                        anyhow::anyhow!("failed to open file `{}`: {}", file_path.display(), err)
                    })?;

                    let metadata = f.metadata().await.map_err(|err| {
                        anyhow::anyhow!(
                            "failed to get metadata of destination file `{}`: {}",
                            file_path.display(),
                            err
                        )
                    })?;

                    let mut buf = Vec::with_capacity(metadata.len() as usize);

                    let mut f = MonitorAsyncAdapter::new(
                        f,
                        file_path.display().to_string(),
                        metadata.len().try_into().unwrap(),
                        Arc::new(Box::new(file_transfer_progress)),
                    );

                    f.read_to_end(&mut buf)
                        .await
                        .map_err(|err| anyhow::anyhow!("failed to read input: {}", err))
                        .map(|_| buf)?
                } else {
                    let mut buf = Vec::new();

                    tokio::io::stdin()
                        .read_to_end(&mut buf)
                        .await
                        .map_err(|err| anyhow::anyhow!("failed to read input: {}", err))?;

                    buf
                };

                provider
                    .write(&buf)
                    .await
                    .map_err(|err| anyhow::anyhow!("failed to write asset: {}", err))
            };

            let res = futures::join!(copy, transfer_join);

            let id = res.0?;
            res.1?;

            println!("{}", id);
        }
        Commands::Explain {
            identifier,
            show_data,
        } => {
            match &identifier {
                Identifier::Data(data) => {
                    println!(
                        "{} is a {} identifier.",
                        style(&identifier).bold().yellow(),
                        style("data").bold().cyan()
                    );
                    println!(
                        "It's data is {} byte(s) long and is contained in the identifier itself.",
                        style(data.len()).bold().green()
                    );
                }
                Identifier::HashRef(id) => {
                    println!(
                        "{} is a {} identifier.",
                        style(&identifier).bold().yellow(),
                        style("hash-ref").bold().cyan()
                    );
                    println!(
                        "It's data is {} byte(s) long and should live directly in a content-store blob.",
                        style(id.data_size()).bold().green()
                    );
                }
                Identifier::ManifestRef(size, id) => {
                    println!(
                        "{} is a {} identifier.",
                        style(&identifier).bold().yellow(),
                        style("manifest-ref").bold().cyan()
                    );
                    println!(
                        "It's data is {} byte(s) long and is split across several identifiers in the content-store. The manifest at `{}` describes the data.",
                        style(size).bold().green(),
                        style(id.to_string()).bold().yellow()
                    );
                }
                Identifier::Alias(key) => {
                    println!(
                        "{} is an {} identifier.",
                        style(&identifier).bold().yellow(),
                        style("alias").bold().cyan()
                    );

                    match String::from_utf8(key.to_vec()) {
                        Ok(key) => {
                            println!("It has an UTF-8 key: `{}`.", style(key).bold().green());
                        }
                        Err(_) => println!(
                            "It has a non-UTF-8 key: `{}`.",
                            style(hex::encode(key)).bold().green()
                        ),
                    }

                    match provider.resolve_alias(key).await {
                        Ok(id) => println!(
                            "It points to the identifier: `{}`.",
                            style(&id).bold().yellow()
                        ),
                        Err(Error::IdentifierNotFound(_)) => {
                            println!("The alias does not seem to exist.");
                        }
                        Err(err) => {
                            println!("Resolving of the alias failed: {}", style(err).bold().red());
                        }
                    }
                }
            };

            match provider.get_reader(&identifier).await {
                Ok(mut reader) => {
                    println!(
                        "The data comes from: {}",
                        style(reader.origin()).bold().magenta()
                    );

                    if reader.size() == 0 {
                        println!("The identifier points to an empty blob.");
                    } else if show_data {
                        let mut buf = Vec::new();
                        const MAX_BUF_SIZE: usize = 512;
                        buf.reserve(min(MAX_BUF_SIZE, reader.size()));

                        if reader.size() < MAX_BUF_SIZE {
                            reader.read_to_end(&mut buf).await?;
                        } else {
                            println!("The content is too large to be printed in its integrality: only showing up the first {} bytes.", style(MAX_BUF_SIZE).bold().cyan());
                            reader
                                .take(MAX_BUF_SIZE.try_into().unwrap())
                                .read_to_end(&mut buf)
                                .await?;
                        };

                        match std::str::from_utf8(&buf) {
                            Ok(s) => println!("It's data is valid UTF-8:\n{}", s),
                            Err(_) => {
                                println!("It's data is not UTF-8:\n{}", hex::encode(&buf));
                            }
                        };
                    } else {
                        println!(
                            "The content is not shown. Specify `{}` to show it.",
                            style("--show-data").bold().cyan()
                        );
                    }
                }
                Err(Error::IdentifierNotFound(_)) => {
                    println!(
                        "{}: the content-store does not contain the identifier.",
                        style("Error").bold().red()
                    );
                }
                Err(err) => {
                    println!("Failed to get a reader: {}", style(err).bold().red());
                }
            }
        }
    }

    Ok(())
}
