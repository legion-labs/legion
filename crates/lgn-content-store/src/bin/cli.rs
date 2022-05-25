//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

use std::{
    cmp::min,
    collections::{BTreeSet, HashMap, HashSet},
    mem::size_of,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, RwLock},
};

use async_recursion::async_recursion;
use clap::{Parser, Subcommand};
use console::style;
use futures::Future;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lgn_content_store::{
    indexing::{
        BasicIndexer, CompositeIndexer, IndexKey, IndexKeyDisplayFormat, IndexableResource,
        ResourceReader, ResourceWriter, StaticIndexer, Tree, TreeIdentifier, TreeLeafNode,
        TreeNode, TreeReader, TreeWriter,
    },
    Config, Error, HashRef, Identifier, MonitorAsyncAdapter, Provider, TransferCallbacks,
};
use lgn_telemetry_sink::TelemetryGuardBuilder;
use lgn_tracing::{async_span_scope, LevelFilter};
use serde::{Deserialize, Serialize};
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
    Tree {
        #[clap(subcommand)]
        command: TreeCommands,
    },
}

#[derive(Subcommand, Debug)]
enum TreeCommands {
    Show {
        identifier: TreeIdentifier,
        #[clap(
            short = 'r',
            long = "recursion-level",
            help = "The recursion level to use",
            default_value_t = 0
        )]
        recursion_level: u32,

        #[clap(
            short = 'f',
            long = "display-format",
            help = "The format to use to display index keys",
            default_value_t = IndexKeyDisplayFormat::Hex
        )]
        display_format: IndexKeyDisplayFormat,
    },
    BuildSearchIndex {
        #[clap(help = "The words file to build the tree from")]
        file_path: PathBuf,
        #[clap(
            long = "min-sequence-length",
            help = "The minimum length of sequences to index",
            default_value_t = 3
        )]
        min_sequence_length: u32,
        #[clap(
            long = "max-sequence-length",
            help = "The maximum length of sequences to index",
            default_value_t = 5
        )]
        max_sequence_length: u32,
    },
    Search {
        #[clap(help = "The tree root identifier")]
        tree_id: TreeIdentifier,

        #[clap(help = "The sequence to search for")]
        sequence: String,
    },
}

#[derive(Debug)]
struct TransferProgress {
    progress: Arc<MultiProgress>,
    progress_style: ProgressStyle,
    bars: Arc<RwLock<HashMap<String, ProgressBar>>>,
}

impl TransferProgress {
    fn new() -> Self {
        let progress = Arc::new(MultiProgress::new());
        let progress_style = ProgressStyle::default_bar()
                .template("{prefix:52!} [{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}");
        let bars = Arc::new(RwLock::new(HashMap::new()));

        Self {
            progress,
            progress_style,
            bars,
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
        if let Some(bar) = self.bars.write().unwrap().remove(&id.to_string()) {
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
        if let Some(bar) = self.bars.write().unwrap().remove(id) {
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
    let transfer_progress = Arc::new(TransferProgress::new());
    let transfer_join = transfer_progress.join();

    match args.command {
        Commands::Get {
            identifier,
            file_path,
        } => {
            provider.set_download_callbacks(transfer_progress.clone());

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
                Arc::new(Box::new(transfer_progress)),
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
            provider.set_upload_callbacks(transfer_progress.clone());

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
                        Arc::new(Box::new(transfer_progress)),
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
        Commands::Tree { command } => match command {
            TreeCommands::Show {
                identifier,
                recursion_level,
                display_format,
            } => {
                let tree = read_tree(
                    &provider,
                    "root",
                    &identifier,
                    display_format,
                    recursion_level,
                )
                .await;
                println!("{}", tree);
            }
            TreeCommands::BuildSearchIndex {
                file_path,
                min_sequence_length,
                max_sequence_length,
            } => {
                if min_sequence_length > max_sequence_length {
                    return Err(anyhow::anyhow!(
                        "min_sequence_length must be less than or equal to max_sequence_length"
                    ));
                }

                // Too verbose.
                //provider.set_upload_callbacks(transfer_progress.clone());

                let all_words = &tokio::fs::read_to_string(file_path)
                    .await?
                    .split('\n')
                    .filter_map(|line| {
                        let line = line.trim().to_lowercase();

                        if line.len() >= min_sequence_length.try_into().unwrap() {
                            Some(line)
                        } else {
                            None
                        }
                    })
                    .collect::<BTreeSet<_>>();

                let provider = Arc::new(provider);

                let mut indexes = Vec::new();

                for sequence_length in min_sequence_length..=max_sequence_length {
                    let provider = Arc::clone(&provider);
                    let bar_id = {
                        let bar = transfer_progress
                            .progress
                            .add(ProgressBar::new(all_words.len().try_into().unwrap()));
                        let progress_style = ProgressStyle::default_bar()
                .template("{prefix:32!} [{elapsed_precise}] {bar:40.yellow/blue} {pos}/{len} ({percent}%, {eta}) {msg}");
                        bar.set_style(progress_style);
                        bar.set_prefix(format!("{} chars seq.", sequence_length));

                        let bar_id = format!("seq-{}", sequence_length);

                        transfer_progress
                            .bars
                            .write()
                            .unwrap()
                            .insert(bar_id.clone(), bar);

                        bar_id
                    };
                    let transfer_progress = transfer_progress.clone();

                    let index: Pin<Box<dyn Future<Output = anyhow::Result<_>>>> =
                        Box::pin(async move {
                            let provider = provider.begin_transaction_in_memory();
                            let mut tree_id = provider.write_tree(&Tree::default()).await?;

                            let seq_len = sequence_length.try_into().unwrap();
                            let indexer = StaticIndexer::new(seq_len);
                            let mut indexed_count = 0;

                            for word in all_words {
                                indexed_count += 1;

                                if indexed_count % 10000 == 0 {
                                    transfer_progress
                                        .bars
                                        .read()
                                        .unwrap()
                                        .get(&bar_id)
                                        .unwrap()
                                        .set_position(indexed_count.try_into().unwrap());
                                }

                                if word.len() < seq_len {
                                    continue;
                                }

                                for i in 0..=(word.len() - seq_len) {
                                    let seq = word[i..i + seq_len].as_bytes();

                                    if seq.len() != seq_len {
                                        continue;
                                    }

                                    let index_key = seq.into();
                                    match indexer.get_leaf(&provider, &tree_id, &index_key).await? {
                                        Some(leaf) => {
                                            let mut words: Words = provider
                                                .read_resource(&leaf.unwrap_resource())
                                                .await?;

                                            words.0.insert(word.clone());

                                            let leaf = TreeLeafNode::Resource(
                                                provider.write_resource(&words).await?,
                                            );

                                            (tree_id, _) = indexer
                                                .replace_leaf(&provider, &tree_id, &index_key, leaf)
                                                .await?;
                                        }
                                        None => {
                                            let mut words = Words::default();
                                            words.0.insert(word.clone());

                                            let leaf = TreeLeafNode::Resource(
                                                provider.write_resource(&words).await?,
                                            );

                                            tree_id = indexer
                                                .add_leaf(&provider, &tree_id, &index_key, leaf)
                                                .await?;
                                        }
                                    }
                                }
                            }

                            if let Err((_, err)) = provider.commit_transaction().await {
                                return Err(err.into());
                            };

                            transfer_progress
                                .bars
                                .read()
                                .unwrap()
                                .get(&bar_id)
                                .unwrap()
                                .finish_with_message("✔️");

                            Ok((sequence_length, bar_id, tree_id))
                        });

                    indexes.push(index);
                }

                let index: Pin<Box<dyn Future<Output = anyhow::Result<_>>>> =
                    Box::pin(async move {
                        let res = futures::future::join_all(indexes)
                            .await
                            .into_iter()
                            .collect::<anyhow::Result<Vec<_>>>()?;

                        let mut root_tree_id = provider.write_tree(&Tree::default()).await?;
                        let indexer = StaticIndexer::new(size_of::<u32>());

                        for (sequence_length, bar_id, tree_id) in res {
                            transfer_progress.bars.write().unwrap().remove(&bar_id);

                            root_tree_id = indexer
                                .add_leaf(
                                    &provider,
                                    &root_tree_id,
                                    &sequence_length.into(),
                                    TreeLeafNode::TreeRoot(tree_id),
                                )
                                .await?;
                        }

                        Ok(root_tree_id)
                    });

                let (res, _) = futures::join!(index, transfer_join);

                let tree_id = res?;

                println!("{}", tree_id);
            }
            TreeCommands::Search { tree_id, sequence } => {
                let sequence_length = sequence.len();
                let indexer = CompositeIndexer::new(
                    StaticIndexer::new(size_of::<u32>()),
                    StaticIndexer::new(sequence_length),
                );
                let seq_len: u32 = sequence.len().try_into().unwrap();
                let index_key = IndexKey::compose(seq_len, sequence.as_bytes());

                match indexer.get_leaf(&provider, &tree_id, &index_key).await? {
                    Some(leaf) => {
                        let words: Words = provider.read_resource(&leaf.unwrap_resource()).await?;

                        for word in &words.0 {
                            println!("{}", word);
                        }
                    }
                    None => {
                        eprintln!("No results found.");
                    }
                }
            }
        },
    }

    Ok(())
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Words(HashSet<String>);

impl IndexableResource for Words {}

#[async_recursion]
async fn read_tree(
    provider: &Provider,
    name: &str,
    id: &TreeIdentifier,
    display_format: IndexKeyDisplayFormat,
    recursion_level: u32,
) -> termtree::Tree<String> {
    match provider.read_tree(id).await {
        Ok(tree) => {
            let mut r = termtree::Tree::new(format!(
                "{} (tree: {}): count: {} - total_size: {}",
                name,
                id,
                tree.count(),
                tree.total_size()
            ));

            for (k, n) in tree.children() {
                let name = k.format(display_format);

                match n {
                    TreeNode::Branch(id) => {
                        if recursion_level > 0 {
                            r.push(
                                read_tree(provider, &name, id, display_format, recursion_level - 1)
                                    .await,
                            );
                        } else {
                            r.push(format!("{} (tree: {})", name, id));
                        }
                    }
                    TreeNode::Leaf(n) => match n {
                        TreeLeafNode::Resource(id) => {
                            r.push(format!("{} (resource: {})", name, id));
                        }
                        TreeLeafNode::TreeRoot(id) => {
                            let name = format!("{} (subtree)", name);
                            if recursion_level > 0 {
                                r.push(
                                    read_tree(
                                        provider,
                                        &name,
                                        id,
                                        display_format,
                                        recursion_level - 1,
                                    )
                                    .await,
                                );
                            } else {
                                r.push(format!("{} (tree: {})", name, id));
                            }
                        }
                    },
                };
            }

            r
        }
        Err(err) => termtree::Tree::new(format!(
            "{} (tree: {}): failed to read tree: {}",
            name, id, err,
        )),
    }
}
