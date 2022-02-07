//! Data scraping utility.
//!
//! Provides functionalities that help diagnose project's data.
//!
//! The diagnostic tools in this binary include:
//! * **rty** - `ResourceType` <-> String lookup utility.
//! * **source** - `ResourceId` <-> `ResourcePathName` lookup utility.
//! * **asset** - Asset file header output.
//! * **explain** - Prints `ResourcePathId`-related details of a specified
//!   `ResourceId`.
//! * **graph** - Prints a build graph of a specified resource (`ResourcePathId`
//!   or `ResourceId`) in Graphviz DOT format.
//!
//! # `rty` - Resource Type Tool
//!
//! Various commands to help identify resource types known under a specified
//! source code directory.
//!
//! It helps with mapping between `Resource::TYPENAME` and `Resource::TYPE`.
//!
//! ### List resource types
//!
//! List all **resource types** found in code under a specified directory:
//!
//! ```text
//! $ data-scrape rty lib\ list
//!
//! runtime_texture = f9c9670d
//! runtime_material = 669eb7d0
//! offline_texture = 74dc0e53
//! psd = 13b5a84e
//! ```
//!
//! ### Encode `Resource::TYPENAME`
//!
//! Show the hashed value of a given resource type name (in code under a
//! specified directory).
//!
//! ```text
//! $ data-scrape rty lib\ encode psd
//!
//! psd = 13b5a84e
//! ```
//!
//! ### Decode `Resource::TYPE`
//!
//! Show a human readable name of a given resource hash (in code under a
//! specified directory).
//!
//! ```text
//! $ data-scrape rty lib\ decode 13b5a84e
//!
//! psd = 13b5a84e
//! ```
//!
//! # `source` - Resource Name Tool
//!
//! Various commands to help translate between source resource's pathname and
//! its id.
//!
//! It helps with mapping between `ResourceId` and `ResourcePathName` of a
//! **source resource**.
//!
//! ### List resources
//!
//! List all **source resources** under a specified project:
//!
//! ```text
//! $ data-scrape source .\test\sample-data\ list
//!
//! /image/ground.psd = 13b5a84e00000000d9e5871a48bd55c5
//! /world/sample_1/ground.mat = 2b368fed000000004e9d6dcb039451e3
//! /world/sample_1/cube_3.ent.ins = 417862e50000000054537dedac8437c4
//! ...
//! ```
//!
//! ### Find id of a resource
//!
//! Show the id of the resource under a specified pathname.
//!
//! ```text
//! $ data-scrape source .\test\sample-data\ id /world/sample_1/cube_1.ent.ins
//!
//! /world/sample_1/cube_1.ent.ins = 417862e500000000e9c81a578a265cda
//! ```
//!
//! ### Find pathname of specified resource id.
//!
//! ```text
//! $ data-scrape source .\test\sample-data\ name 417862e500000000e321168f3653db42
//!
//! /prefab/props/cube_group_cube_1.ent.ins = 417862e500000000e321168f3653db42
//! ```
//!
//! # `asset` - Asset file header display tool
//!
//! Display the header content information (resource type, content size, etc)
//! from either a single asset file, or a directory of asset files.
//!
//! ```text
//! $ data-scrape asset .\test\sample-data\temp\a88c4baf56023f98e12508ae2c4488c9
//!
//! asset a88c4baf56023f98e12508ae2c4488c9
//!         file type: asft, version: 1
//!         asset type: 019c8223
//!         asset count: 1
//!         asset size: 209
//!
//! $data-scrape asset .\test\sample-data\temp
//!
//! asset 108608222a9c9987a5589c2285b6115d
//!         raw asset file
//!         asset content size: 159
//!
//! asset 12fb356494c92be198a504ba2e915978
//!         file type: asft, version: 1
//!         asset type: 74dc0e53
//!         asset count: 1
//!         asset content size: 2500634
//!
//! asset 13955f2e5e320e0002d36bed544d34ee
//!         file type: asft, version: 1
//! ...
//! ```
//!
//! # `explain` - Tool that shows details about specified `ResourceId` or
//! `ResourcePathId`
//!
//! The tool prints detailed information about the specified `ResourcePathId` -
//! such as the name of the source resource, its type and all transformations
//! with their parameters.
//!
//! It also accepts `ResourceId` as input - in which case it will try to find
//! its corresponding `ResourcePathId` (if it occurred during data compilation).
//!
//! ## `ResourceId` as input
//!
//! ```text
//! $ data-scrape explain ea2510e8000000007354d8806dd36e41
//!
//! Explained:      /world/sample_1/ground.mesh (offline_mesh) => runtime_mesh
//! ResourcePathId: 4e1dd441000000007714d9ad404557f7|ea2510e8
//! ```
//!
//! ## `ResourcePathId` as input
//!
//! ```text
//! $ data-scrape explain '13b5a84e0000000061f6de1be4764697|74dc0e53_albedo|f9c9670d'
//!
//! Explained:      /image/ground.psd (psd) => offline_texture('albedo') => runtime_texture
//! ResourcePathId: 13b5a84e0000000061f6de1be4764697|74dc0e53_albedo|f9c9670d
//! ```
//!
//! **NOTE**: It requires running `data-scrape configure` first.
//!
//!  # `graph` - Tool that prints a build graph in Graphviz DOT format
//!
//! Prints a build graph of a specified resources (`ResourcePathId` or `ResourceId`) in a [Graphviz DOT](https://www.graphviz.org/doc/info/lang.html) format.
//!
//! ```text
//! $ data-scrape graph d004cd1c00000000fcd3242ec9691beb
//!
//! digraph {
//! 0 [ label = "d004cd1c00000000fcd3242ec9691beb" label = "/world/sample_1.ent => offline_entity"]
//! 1 [ label = "417862e5000000005127bdca42145845|0cd05ad8" label = "/world/sample_1/cube_group_1.ent.ins => offline_instance => runtime_instance"]
//! 2 [ label = "417862e5000000007a80bfadcb6549e7|0cd05ad8" label = "/world/sample_1/cube_2.ent.ins => offline_instance => runtime_instance"]
//! 3 [ label = "417862e5000000007df5d0fcdb1eb69b|0cd05ad8" label = "/world/sample_1/cube_3.ent.ins => offline_instance => runtime_instance"]
//! ...
//! ```
//!
//! # `configure` - Scrape tool configuration.
//!
//! Creates a configuration file used by `data-scrape` tool to easily find a
//! project file, build index and cache resource type names.
//!
//! By default it will point to project and build index located in
//! **sample-data** directory and to the legion workspace's code directories.
//!
//! ```text
//! $ data-scrape configure
//!
//! Configuration '"D:\\github.com\\legion-labs\\legion\\target\\debug\\scrape-config.json"' created!
//!     Code Paths: ["D:\\github.com\\legion-labs\\legion\\lib\\", "D:\\github.com\\legion-labs\\legion\\client\\", "D:\\github.com\\legion-labs\\legion\\test\\"].
//!     Project: "D:\\github.com\\legion-labs\\legion\\test\\sample-data\\".
//!     Build Index: "D:\\github.com\\legion-labs\\legion\\test\\sample-data\\temp\\".
//!     Resource Type Count: 20.
//! ```

// crate-specific lint exceptions:
//#![allow()]

use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use byteorder::{LittleEndian, ReadBytesExt};
use clap::{AppSettings, Parser, Subcommand};
use lgn_content_store::Checksum;
use lgn_data_offline::{
    resource::{Project, ResourcePathName},
    ResourcePathId,
};
use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};

mod config;
use config::Config;

#[derive(Parser, Debug)]
#[clap(name = "Data Scraper")]
#[clap(about = "Data scraping utility", version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parse code for ResourceType information
    #[clap(name = "rty")]
    Rty {
        /// Path in to code root.
        path: Option<PathBuf>,
        #[clap(subcommand)]
        command: RtyCommands,
    },
    /// Explain
    #[clap(name = "explain")]
    Explain {
        /// Id to explain
        id: String,
    },
    /// Parse project index for source resource information
    #[clap(name = "source")]
    Source {
        /// Path to directory containing the project
        path: Option<PathBuf>,
        #[clap(subcommand)]
        command: SourceCommands,
    },
    /// Parse asset file, or folder, to extract asset meta-data
    #[clap(name = "asset")]
    Asset {
        /// Path to single asset file, or directory containing several asset files
        path: PathBuf,
    },
    /// Print Graphviz representation of a build graph in DOT format
    #[clap(name = "graph")]
    Graph {
        /// Compile path (either ResourcePathId or ResourceId) to print the graph of
        id: String,
    },
    /// Creates a configuration file containing paths to relevant locations
    #[clap(name = "configure")]
    Configure {
        /// Paths to code directories to scan for resource types
        #[clap(long, use_delimiter = true)]
        code_path: Option<Vec<PathBuf>>,
        /// Path to project index to be able to resolve ResourcePathName
        #[clap(long)]
        project: Option<PathBuf>,
        /// Path to build index to be able to resolve ResourcePathId
        #[clap(long = "buildindex")]
        build_index: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum RtyCommands {
    /// List all resource types in the project
    #[clap(name = "list")]
    List,
    /// Encodes human readable resource name to hash value.
    #[clap(name = "encode")]
    Encode {
        /// Human readable resource name - Resource::TYPENAME
        name: String,
    },
    /// Decodes hash value of resource type to human readable name.
    #[clap(name = "decode")]
    Decode {
        /// ResourceType hash - Resource::TYPE
        ty: String,
    },
}

#[derive(Subcommand, Debug)]
enum SourceCommands {
    /// List all source resource information
    #[clap(name = "list")]
    List,
    /// Finds a name of a given ResourceId
    #[clap(name = "name")]
    Name {
        /// ResourceId to find a name of
        id: String,
    },
    /// Find the id of a given ResourcePathName
    #[clap(name = "id")]
    Id {
        /// ResourcePathName to find id of
        name: String,
    },
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Cli::parse();

    //
    // try opening the configuration file first.
    //
    let config = Config::read(Config::default_path()).ok();

    match args.command {
        Commands::Rty { path, command } => {
            let code_dir = path.unwrap_or_else(|| std::env::current_dir().unwrap());
            match command {
                RtyCommands::List => {
                    for (name, ty) in ResourceTypeIterator::new(find_files(&code_dir, &["rs"])) {
                        println!("{} = {}", name, ty);
                    }
                }
                RtyCommands::Encode { name } => {
                    if let Some((name, ty)) =
                        ResourceTypeIterator::new(find_files(&code_dir, &["rs"]))
                            .find(|(n, _)| *n == name)
                    {
                        println!("{} = {}", name, ty);
                    } else {
                        println!("{} = {}", name, ResourceType::new(name.as_bytes()));
                    }
                }
                RtyCommands::Decode { ty } => {
                    let searched_ty = ResourceType::from_str(&ty).unwrap();
                    if let Some((name, ty)) =
                        ResourceTypeIterator::new(find_files(&code_dir, &["rs"]))
                            .find(|(_, ty)| ty == &searched_ty)
                    {
                        println!("{} = {}", name, ty);
                    }
                }
            }
        }
        Commands::Explain { id } => {
            if let Some(config) = config {
                let (build, project) = config.open().await?;
                let rid = {
                    if let Ok(rid) = ResourcePathId::from_str(&id) {
                        rid
                    } else if let Ok(resource_id) = id.parse() {
                        if let Some(rid) = build.lookup_pathid(resource_id) {
                            rid
                        } else {
                            return Err(format!(
                                "Failed to find a source ResourcePathId for ResourceId '{}'",
                                resource_id
                            ));
                        }
                    } else {
                        return Err(format!("Failed to parse id: '{}'", id));
                    }
                };

                let pretty = pretty_name_from_pathid(&rid, &project, &config);
                println!("Explained: \t{}", pretty);
                println!("ResourcePathId: {}", rid);
            } else {
                return Err(
                    "Configuration not found. Run 'data-scrape configure' first.".to_string(),
                );
            }
        }
        Commands::Source { path, command } => {
            let proj_file = path.unwrap_or_else(|| std::env::current_dir().unwrap());
            let project = Project::open(proj_file).await.map_err(|e| e.to_string())?;
            match command {
                SourceCommands::List => {
                    for id in project.resource_list().await {
                        let name = project.resource_name(id).map_err(|e| e.to_string())?;
                        println!("{} = {}", name, id);
                    }
                }
                SourceCommands::Name { id } => {
                    let type_id = id.parse::<ResourceTypeAndId>().map_err(|e| e.to_string())?;
                    if let Ok(name) = project.resource_name(type_id.id) {
                        println!("{} = {}", name, type_id);
                    } else {
                        println!("None");
                    }
                }
                SourceCommands::Id { name } => {
                    let name = ResourcePathName::from(name);
                    if let Ok(id) = project.find_resource(&name).await {
                        println!("{} = {}", name, id);
                    } else {
                        println!("None");
                    }
                }
            }
        }
        Commands::Asset { path } => {
            if path.is_file() {
                parse_asset_file(path, &config).await;
            } else if path.is_dir() {
                for entry in path.read_dir().unwrap().flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        parse_asset_file(path, &config).await;
                    }
                }
            }
        }
        Commands::Graph { id } => {
            if let Some(config) = config {
                let (build, project) = config.open().await?;
                let rid = {
                    if let Ok(resource_id) = id.parse() {
                        build
                            .lookup_pathid(resource_id)
                            .ok_or(format!("ResourceId '{}' not found", resource_id))?
                    } else {
                        ResourcePathId::from_str(&id)
                            .map_err(|_e| format!("Invalid ResourcePathId '{}'", id))?
                    }
                };
                let output = build
                    .print_build_graph(rid, |rid| pretty_name_from_pathid(rid, &project, &config));
                println!("{}", output);
            } else {
                return Err(
                    "Configuration not found. Run 'data-scrape configure' first.".to_string(),
                );
            }
        }
        Commands::Configure {
            code_path,
            project,
            build_index,
        } => {
            let config_path = Config::default_path();
            let workspace_dir = Config::workspace_dir();

            let code_paths = code_path.unwrap_or_else(|| {
                vec![workspace_dir.join("crates/"), workspace_dir.join("tests/")]
            });

            let build_index =
                build_index.unwrap_or_else(|| workspace_dir.join("tests/sample-data/temp/"));

            let project =
                project.unwrap_or_else(|| workspace_dir.join("tests/sample-data/"));

            let type_map = {
                let mut t = BTreeMap::<ResourceType, String>::new();
                for dir in &code_paths {
                    for (name, ty) in ResourceTypeIterator::new(find_files(&dir, &["rs"])) {
                        t.insert(ty, name);
                    }
                }
                t
            };

            let config = Config {
                code_paths,
                project,
                buildindex: build_index,
                type_map,
            };

            config
                .write(&config_path)
                .map_err(|e| format!("failed with: '{}'", e))?;

            println!("Configuration '{:?}' created!", config_path);
            println!("\tCode Paths: {:?}.", config.code_paths);
            println!("\tProject: {:?}.", config.project);
            println!("\tBuild Index: {:?}.", config.buildindex);
            println!("\tResource Type Count: {}.", config.type_map.len());
        }
    }
    Ok(())
}

struct ResourceTypeIterator {
    files: Vec<PathBuf>,
    types: Vec<(String, ResourceType)>,
}

impl ResourceTypeIterator {
    fn new(files: Vec<PathBuf>) -> Self {
        Self {
            files,
            types: vec![],
        }
    }
}

impl Iterator for ResourceTypeIterator {
    type Item = (String, ResourceType);

    fn next(&mut self) -> Option<Self::Item> {
        while self.types.is_empty() {
            if let Some(next_file) = self.files.pop() {
                self.types = all_declared_resources(&next_file);
            } else {
                return None;
            }
        }
        if let Some(next_type) = self.types.pop() {
            return Some(next_type);
        }
        None
    }
}

// Recursively searches all directories in `dir` for files with `extensions`
fn find_files(dir: impl AsRef<Path>, extensions: &[&str]) -> Vec<PathBuf> {
    let dir = dir.as_ref();

    let mut files = vec![];

    for entry in dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.append(&mut find_files(&path, extensions));
        } else if let Some(ext) = path.extension().and_then(std::ffi::OsStr::to_str) {
            if extensions.contains(&ext) {
                files.push(path);
            }
        }
    }

    files
}

// recursively parses modules in search of resource attributes
fn find_resource_attribs(content: &[syn::Item]) -> Vec<(String, ResourceType)> {
    let get_resource_name = |a: &syn::ItemStruct| -> Option<(String, ResourceType)> {
        for a in &a.attrs {
            if a.path.segments.len() == 1 && a.path.segments[0].ident == "resource" {
                if let Ok(arg) = a.parse_args::<syn::LitStr>() {
                    let arg = arg.value();

                    let ty = ResourceType::new(arg.as_bytes());

                    return Some((arg, ty));
                }
            }
        }
        None
    };

    let mut types = vec![];
    for item in content {
        if let syn::Item::Struct(struc) = &item {
            if let Some((name, ty)) = get_resource_name(struc) {
                types.push((name, ty));
            }
        } else if let syn::Item::Mod(syn::ItemMod {
            content: Some((_, c)),
            ..
        }) = &item
        {
            types.extend(find_resource_attribs(c));
        }
    }
    types
}

// Finds all #[resource("name")] attributes in a file and returns (name, hashed
// name) tuple.
fn all_declared_resources(source: &Path) -> Vec<(String, ResourceType)> {
    let src = std::fs::read_to_string(&source).expect("Read file");
    // Quickly bail out without parsing the file
    if src.find("#[resource(") == None {
        return vec![];
    }
    proc_macro2::fallback::force(); // prevent panic, if panic = abort is set
    let tokens = proc_macro2::TokenStream::from_str(&src).expect("Tokenize source file");
    let ast: syn::File = syn::parse2(tokens).expect("Unable to parse file");
    find_resource_attribs(&ast.items)
}

// Reads an asset file, and prints out its header information.
#[allow(unsafe_code)]
async fn parse_asset_file(path: impl AsRef<Path>, config: &Option<Config>) {
    let path = path.as_ref();
    let mut f = File::open(path).expect("unable to open asset file");

    let file_name = path.file_name().unwrap().to_string_lossy();
    let checksum = file_name.parse::<Checksum>();
    if let Err(_e) = checksum {
        // not an asset file, just ignore it
        return;
    }
    let checksum = checksum.unwrap();
    println!("\nasset {}", checksum);

    let mut typename: [u8; 4] = [0; 4];
    let typename_result = f.read_exact(&mut typename);
    const ASSET_FILE_TYPENAME: &[u8; 4] = b"asft";
    if typename_result.is_err() || &typename != ASSET_FILE_TYPENAME {
        println!("\traw asset file");
        let metadata = std::fs::metadata(path).expect("failed to read metadata");
        println!("\tasset content size: {}", metadata.len());
        return;
    }

    let version = f.read_u16::<LittleEndian>().expect("valid data");
    if version != 1 {
        println!("\tunsupported asset file version");
        return;
    }

    let typename = std::str::from_utf8(&typename).unwrap();
    println!("\tfile type: {}, version: {}", typename, version);

    let reference_count = f.read_u64::<LittleEndian>().expect("valid data");
    if reference_count != 0 {
        println!("\treference count: {}", reference_count);
        for _ in 0..reference_count {
            let asset_ref_type =
                ResourceType::from_raw(f.read_u64::<LittleEndian>().expect("valid data"));
            let asset_ref_id =
                ResourceId::from_raw(f.read_u128::<LittleEndian>().expect("valid data"));
            if let Some(config) = config {
                let (_build, project) = config.open().await.expect("open config");
                let path_id = ResourcePathId::from(ResourceTypeAndId {
                    kind: asset_ref_type,
                    id: asset_ref_id,
                });
                println!(
                    "\t\treference: {}",
                    pretty_name_from_pathid(&path_id, &project, config)
                );
            } else {
                println!(
                    "\t\treference: {}",
                    ResourceTypeAndId {
                        kind: asset_ref_type,
                        id: asset_ref_id
                    }
                );
            }
        }
    }

    let asset_type = ResourceType::from_raw(f.read_u64::<LittleEndian>().expect("valid data"));
    if let Some(config) = config {
        if let Some(asset_type_name) = config.type_map.get(&asset_type).cloned() {
            println!("\tasset type: {} ({})", asset_type, asset_type_name);
        } else {
            println!("\tasset type: {}", asset_type);
        }
    } else {
        println!("\tasset type: {}", asset_type);
    }

    let asset_count = f.read_u64::<LittleEndian>().expect("valid data");
    println!("\tasset count: {}", asset_count);

    let nbytes = f.read_u64::<LittleEndian>().expect("valid data");
    println!("\tasset content size: {}", nbytes);
}

fn pretty_name_from_pathid(rid: &ResourcePathId, project: &Project, config: &Config) -> String {
    let mut output_text = String::new();

    if let Ok(source_name) = project.resource_name(rid.source_resource().id) {
        output_text.push_str(&source_name.to_string());
    } else {
        output_text.push_str(&rid.source_resource().to_string());
    }

    let source_ty_pretty = config
        .type_map
        .get(&rid.source_resource().kind)
        .cloned()
        .unwrap_or_else(|| rid.source_resource().kind.to_string());
    output_text.push_str(&format!(" ({})", source_ty_pretty));

    for (_, target, name) in rid.transforms() {
        let target_ty_pretty = config
            .type_map
            .get(&target)
            .cloned()
            .unwrap_or_else(|| target.to_string());
        output_text.push_str(&format!(" => {}", target_ty_pretty));
        if let Some(name) = name {
            output_text.push_str(&format!("('{}')", name));
        }
    }

    output_text
}
