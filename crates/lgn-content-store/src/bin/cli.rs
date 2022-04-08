//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

use std::{
    collections::HashMap,
    fmt::Display,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, RwLock},
};

use bytesize::ByteSize;
use clap::{Parser, Subcommand};
use console::style;
use futures::Future;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lgn_content_store::{
    ChunkIdentifier, ChunkIndex, Chunker, Config, ContentReaderExt, Error, Identifier,
    MonitorAsyncAdapter, MonitorProvider, TransferCallbacks,
};
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{async_span_scope, LevelFilter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,

    #[clap(long="section", short='s', default_value=Config::SECTION_PERSISTENT)]
    section: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Get {
        identifier: ChunkIdentifier,

        file_path: PathBuf,
    },
    Put {
        file_path: Option<PathBuf>,

        #[clap(long, default_value_t = ByteSize::b(Chunker::DEFAULT_CHUNK_SIZE.try_into().unwrap()))]
        chunk_size: ByteSize,
    },
    Explain {
        identifier: GenericIdentifier,
    },
}

#[derive(Debug, Clone)]
enum GenericIdentifier {
    Identifier(Identifier),
    ChunkIdentifier(ChunkIdentifier),
}

impl Display for GenericIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(id) => write!(f, "{}", id),
            Self::ChunkIdentifier(id) => write!(f, "{}", id),
        }
    }
}

impl FromStr for GenericIdentifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(chunk_id) = s.parse::<ChunkIdentifier>() {
            Ok(Self::ChunkIdentifier(chunk_id))
        } else {
            s.parse::<Identifier>().map(GenericIdentifier::Identifier)
        }
    }
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

impl TransferCallbacks<Identifier> for TransferProgress {
    fn on_transfer_avoided(&self, id: &Identifier, total: usize) {
        let bar = self
            .progress
            .add(ProgressBar::new(total.try_into().unwrap()));
        bar.set_style(self.progress_style.clone());
        bar.set_prefix(id.to_string());
        bar.set_position(id.data_size().try_into().unwrap());
        bar.finish_with_message("♥");

        self.bars.write().unwrap().insert(id.to_string(), bar);
    }

    fn on_transfer_started(&self, id: &Identifier, total: usize) {
        let bar = self
            .progress
            .add(ProgressBar::new(total.try_into().unwrap()));
        bar.set_style(self.progress_style.clone());
        bar.set_prefix(id.to_string());

        self.bars.write().unwrap().insert(id.to_string(), bar);
    }

    fn on_transfer_progress(&self, id: &Identifier, _total: usize, inc: usize, _current: usize) {
        if let Some(bar) = self.bars.read().unwrap().get(&id.to_string()) {
            bar.inc(inc.try_into().unwrap());
        }
    }

    fn on_transfer_stopped(
        &self,
        id: &Identifier,
        _total: usize,
        inc: usize,
        _current: usize,
        result: lgn_content_store::Result<()>,
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

impl TransferCallbacks<PathBuf> for TransferProgress {
    fn on_transfer_avoided(&self, _id: &PathBuf, _total: usize) {}

    fn on_transfer_started(&self, id: &PathBuf, total: usize) {
        let bar = self
            .progress
            .add(ProgressBar::new(total.try_into().unwrap()));
        bar.set_style(self.progress_style.clone());
        bar.set_prefix(id.display().to_string());

        self.bars
            .write()
            .unwrap()
            .insert(id.display().to_string(), bar);
    }

    fn on_transfer_progress(&self, id: &PathBuf, _total: usize, inc: usize, _current: usize) {
        if let Some(bar) = self.bars.read().unwrap().get(&id.display().to_string()) {
            bar.inc(inc.try_into().unwrap());
        }
    }

    fn on_transfer_stopped(
        &self,
        id: &PathBuf,
        _total: usize,
        inc: usize,
        _current: usize,
        result: lgn_content_store::Result<()>,
    ) {
        if let Some(bar) = self.bars.read().unwrap().get(&id.display().to_string()) {
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
        .with_local_sink_max_level(LevelFilter::Info)
        .build();

    async_span_scope!("lgn-content-store-srv::main");

    let config = Config::load(&args.section)?;
    let provider = config
        .instantiate_provider()
        .await
        .map_err(|err| anyhow::anyhow!("failed to create content provider: {}", err))?;
    let provider = MonitorProvider::new(provider);

    let transfer_progress = TransferProgress::new();
    let file_transfer_progress = transfer_progress.clone();
    let transfer_join = transfer_progress.join();

    match args.command {
        Commands::Get {
            identifier,
            file_path,
        } => {
            let provider = provider.on_download_callbacks(transfer_progress);
            let chunker = Chunker::default();

            let output = Box::new(tokio::fs::File::create(&file_path).await.map_err(|err| {
                anyhow::anyhow!(
                    "failed to create destination file `{}`: {}",
                    file_path.display(),
                    err
                )
            })?);

            let mut output = MonitorAsyncAdapter::new(
                output,
                file_path,
                identifier.data_size(),
                Arc::new(Box::new(file_transfer_progress)),
            );

            let copy = async move {
                let mut input = chunker
                    .get_chunk_reader(provider, &identifier)
                    .await
                    .map_err(|err| anyhow::anyhow!("failed to get asset: {}", err))?;

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
        Commands::Put {
            file_path,
            chunk_size,
        } => {
            let provider = provider.on_upload_callbacks(transfer_progress);
            let chunker =
                Chunker::default().with_chunk_size(chunk_size.as_u64().try_into().unwrap());

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
                        file_path,
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

                chunker
                    .write_chunk(provider, &buf)
                    .await
                    .map_err(|err| anyhow::anyhow!("failed to write asset: {}", err))
            };

            let res = futures::join!(copy, transfer_join);

            let id = res.0?;
            res.1?;

            println!("{}", id);
        }
        Commands::Explain { identifier } => match identifier {
            GenericIdentifier::ChunkIdentifier(identifier) => {
                println!(
                    "{} is a {} identifier.",
                    style(&identifier).bold().yellow(),
                    style("chunk").bold().cyan()
                );
                println!(
                    "The data it represents to is {} byte(s) long.",
                    style(identifier.data_size()).bold()
                );

                println!(
                    "The chunk manifest it points to is {} bytes long.",
                    style(identifier.content_id().data_size()).bold()
                );

                if identifier.content_id().is_data() {
                    println!(
                        "The chunk manifest is {} in the identifier, which makes the indirection free.",
                        style("inlined").bold().green()
                    );
                } else {
                    let origin = provider.read_origin(identifier.content_id()).await?;

                    println!(
                        "The chunk manifest has id `{}` and comes from {}: {}",
                        style(identifier.content_id()).bold().yellow(),
                        style(origin.name()).bold().red(),
                        style(&origin).cyan(),
                    );
                }

                println!("Fetching the chunk manifest...\n");

                let provider = provider.on_download_callbacks(transfer_progress);
                let chunker = Chunker::default();

                let chunk_index = chunker
                    .read_chunk_index(&provider, &identifier)
                    .await
                    .map_err(|err| anyhow::anyhow!("failed to read chunk index: {}", err))?;

                match chunk_index {
                    ChunkIndex::Linear(identifiers) => {
                        println!(
                            "The chunk manifest is {} and contains {} possibly non-unique identifier(s).",
                            style("linear").bold().blue(),
                            style(identifiers.len()).bold(),
                        );

                        for (i, identifier) in identifiers.iter().enumerate() {
                            let origin = provider.read_origin(identifier).await?;

                            println!(
                                "{:>5}/{}: `{}` is {} byte(s) long and comes from {} ({})",
                                i + 1,
                                identifiers.len(),
                                style(&identifier).bold().yellow(),
                                style(identifier.data_size()).bold(),
                                style(origin.name()).bold().red(),
                                style(&origin).cyan()
                            );
                        }

                        println!("End of the chunk manifest.");
                    }
                }
            }
            GenericIdentifier::Identifier(identifier) => {
                println!(
                    "{} is a {} identifier.",
                    style(&identifier).bold().yellow(),
                    style("raw").bold().cyan()
                );
                println!(
                    "The data it represents to is {} byte(s) long.",
                    style(identifier.data_size()).bold()
                );

                if identifier.is_data() {
                    println!(
                        "The data is {} in the identifier, which makes the indirection free.",
                        style("inlined").bold().green()
                    );
                } else {
                    let origin = provider.read_origin(&identifier).await?;

                    println!(
                        "The data comes from {} ({})",
                        style(origin.name()).bold().red(),
                        style(&origin).cyan(),
                    );
                }
            }
        },
    }

    Ok(())
}
