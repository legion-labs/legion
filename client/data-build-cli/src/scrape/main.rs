//! Data scraping utility.
//!
//! Provides functionalities that help diagnose project's data.
//!
//! The diagnostic tools in this binary include:
//! * **rty** - `ResourceType` <-> String lookup utility.
//! * **source** - `ResourceId` <-> `ResourcePathName` lookup utility.
//! * **asset** - Asset file header output.
//! * **explain** - Prints `ResourcePathId`-related details of a specified `ResourceId`.
//! * **graph** - Prints a build graph of a specified resource (`ResourcePathId` or `ResourceId`) in Graphviz DOT format.
//!
//! # `rty` - Resource Type Tool
//!
//! Various commands to help identify resource types known under a specified sourcecode directory.
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
//! Show the hashed value of a given resource type name (in code under a specified directory).
//!
//! ```text
//! $ data-scrape rty lib\ encode psd
//!
//! psd = 13b5a84e
//! ```
//!
//! ### Decode `Resource::TYPE`
//!
//! Show a human readable name of a given resource hash (in code under a specified directory).
//!
//! ```text
//! $ data-scrape rty lib\ decode 13b5a84e
//!
//! psd = 13b5a84e
//! ```
//!
//! # `source` - Resource Name Tool
//!
//! Various commands to help translate between source resource's pathname and its id.
//!
//! It helps with mapping between `ResourceId` and `ResourcePathName` of a **source resource**.
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
//! Display the header content information (resource type, content size, etc) from
//! either a single asset file, or a directory of asset files.
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
//! # `explain` - Tool that shows details about specified `ResourceId` or `ResourcePathId`
//!
//! The tool prints detailed information about the specified `ResourcePathId` - such as the name of the source resource, its type and all transformations with their parameters.
//!
//! It also accepts `ResourceId` as input - in which case it will try to find its corresponding `ResourcePathId` (if it occured during data compilation).
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
//! Creates a configuration file used by `data-scrape` tool to easily find a project file, build index and cache resource type names.
//!
//! By default it will point to project and build index located in **sample-data** directory and to the legion workspace's code directories.
//!
//! ```text
//! $ data-scrape configure
//!
//! Configuration '"D:\\github.com\\legion-labs\\legion\\target\\debug\\scrape-config.json"' created!
//!     Code Paths: ["D:\\github.com\\legion-labs\\legion\\lib\\", "D:\\github.com\\legion-labs\\legion\\client\\", "D:\\github.com\\legion-labs\\legion\\test\\"].
//!     Project: "D:\\github.com\\legion-labs\\legion\\test\\sample-data\\project.index".
//!     Build Index: "D:\\github.com\\legion-labs\\legion\\test\\sample-data\\temp\\".
//!     Resource Type Count: 20.
//! ```

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow()]

use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use byteorder::{LittleEndian, ReadBytesExt};
use clap::{AppSettings, Arg, SubCommand};
use legion_data_offline::{
    resource::{Project, ResourcePathName},
    ResourcePathId,
};
use legion_data_runtime::{ResourceId, ResourceType};

mod config;
use config::Config;

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), String> {
    let matches = clap::App::new("Data Scraper")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .about("Data scraping utility")
        .subcommand(
            SubCommand::with_name("rty")
                .about("Parse code for ResourceType information")
                .arg(Arg::with_name("path").help("Path in to code root."))
                .subcommand(SubCommand::with_name("list"))
                .subcommand(
                    SubCommand::with_name("encode")
                        .arg(
                            Arg::with_name("name")
                                .help("Human readable resource name - Resource::TYPENAME")
                                .required(true),
                        )
                        .about("Encodes human readable resource name to hash value."),
                )
                .subcommand(
                    SubCommand::with_name("decode")
                        .arg(
                            Arg::with_name("ty")
                                .help("ResourceType hash - Resource::TYPE")
                                .required(true),
                        )
                        .about("Decodes hash value of resource type to human readable name"),
                ),
        )
        .subcommand(
            SubCommand::with_name("explain")
                .arg(Arg::with_name("id").help("Id to explain").required(true)),
        )
        .subcommand(
            SubCommand::with_name("source")
                .about("Parse project index for source resource information")
                .arg(Arg::with_name("path").help("Path to directory containing the project"))
                .subcommand(SubCommand::with_name("list"))
                .subcommand(
                    SubCommand::with_name("name")
                        .arg(
                            Arg::with_name("id")
                                .help("ResourceId to find a name of")
                                .required(true),
                        )
                        .help("Finds a name of a given ResourceId"),
                )
                .subcommand(
                    SubCommand::with_name("id")
                        .arg(
                            Arg::with_name("name")
                                .help("ResourcePathName to find id of")
                                .required(true),
                        )
                        .help("Find the id of a given ResourcePathName"),
                ),
        )
        .subcommand(
            SubCommand::with_name("asset")
                .about("Parse asset file, or folder, to extract asset meta-data")
                .arg(Arg::with_name("path").help(
                    "Path to single asset file, or directory containing several asset files",
                )),
        )
        .subcommand(
            SubCommand::with_name("graph")
                .about("Print Graphviz representation of a build graph in DOT format")
                .arg(Arg::with_name("id").takes_value(true).help(
                    "Compile path (either ResourcePathId or ResourceId) to print the graph of.",
                )),
        )
        .subcommand(
            SubCommand::with_name("configure")
                .about("Creates a configuration file containing paths to relevant locations")
                .arg(
                    Arg::with_name("code_path")
                        .long("code_path")
                        .help("Paths to code directories to scan for resource types")
                        .use_delimiter(true),
                )
                .arg(
                    Arg::with_name("project")
                        .long("project")
                        .takes_value(true)
                        .help("Path to project index to be able to resolve ResourcePathName."),
                )
                .arg(
                    Arg::with_name("buildindex")
                        .long("buildindex")
                        .takes_value(true)
                        .help("Path to build index to be able to resolve ResourcePathId"),
                ),
        )
        .get_matches();

    //
    // try opening the configuration file first.
    //
    let config = Config::read(Config::default_path()).ok();

    if let ("rty", Some(cmd_args)) = matches.subcommand() {
        let code_dir = cmd_args
            .value_of("path")
            .map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

        match cmd_args.subcommand() {
            ("list", _) => {
                for (name, ty) in ResourceTypeIterator::new(find_files(&code_dir, &["rs"])) {
                    println!("{} = {}", name, ty);
                }
            }
            ("encode", Some(cmd_args)) => {
                let searched_name = cmd_args.value_of("name").unwrap();

                if let Some((name, ty)) = ResourceTypeIterator::new(find_files(&code_dir, &["rs"]))
                    .find(|(name, _)| name == searched_name)
                {
                    println!("{} = {}", name, ty);
                }
            }
            ("decode", Some(cmd_args)) => {
                let searched_ty = cmd_args.value_of("ty").unwrap();
                let searched_ty =
                    ResourceType::from_raw(u32::from_str_radix(searched_ty, 16).unwrap());
                if let Some((name, ty)) = ResourceTypeIterator::new(find_files(&code_dir, &["rs"]))
                    .find(|(_, ty)| ty == &searched_ty)
                {
                    println!("{} = {}", name, ty);
                }
            }
            _ => {
                println!("{}", cmd_args.usage());
            }
        }
    } else if let ("source", Some(cmd_args)) = matches.subcommand() {
        let proj_file = cmd_args
            .value_of("path")
            .map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

        let project = Project::open(proj_file).map_err(|e| e.to_string())?;

        match cmd_args.subcommand() {
            ("list", _) => {
                for id in project.resource_list() {
                    let name = project.resource_name(id).map_err(|e| e.to_string())?;
                    println!("{} = {}", name, id);
                }
            }
            ("name", Some(cmd_args)) => {
                let id = cmd_args.value_of("id").unwrap();
                let id = ResourceId::from_str(id).map_err(|e| e.to_string())?;
                if let Ok(name) = project.resource_name(id) {
                    println!("{} = {}", name, id);
                } else {
                    println!("None");
                }
            }
            ("id", Some(cmd_args)) => {
                let name = cmd_args.value_of("name").unwrap();
                let name = ResourcePathName::from(name);
                if let Ok(id) = project.find_resource(&name) {
                    println!("{} = {}", name, id);
                } else {
                    println!("None");
                }
            }
            _ => {
                println!("{}", cmd_args.usage());
            }
        }
    } else if let ("asset", Some(cmd_args)) = matches.subcommand() {
        if let Some(path) = cmd_args.value_of("path") {
            let path = Path::new(path);
            if path.is_file() {
                parse_asset_file(path);
            } else if path.is_dir() {
                for entry in path.read_dir().unwrap().flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        parse_asset_file(path);
                    }
                }
            }
        } else {
            println!("{}", cmd_args.usage());
        }
    } else if let ("explain", Some(cmd_args)) = matches.subcommand() {
        if let Some(config) = config {
            let (build, project) = config.open()?;
            let text_id = cmd_args.value_of("id").unwrap();

            let rid = {
                if let Ok(rid) = ResourcePathId::from_str(text_id) {
                    rid
                } else if let Ok(resource_id) = ResourceId::from_str(text_id) {
                    if let Some(rid) = build.lookup_pathid(resource_id) {
                        rid
                    } else {
                        return Err(format!(
                            "Failed to find a source ResroucePathId for ResourceId '{}'",
                            resource_id
                        ));
                    }
                } else {
                    return Err(format!("Failed to parse id: '{}'", text_id));
                }
            };

            let pretty = pretty_name_from_pathid(&rid, &project, &config);
            println!("Explained: \t{}", pretty);
            println!("ResourcePathId: {}", rid);
        } else {
            return Err("Configuration not found. Run 'data-scrape configure' first.".to_string());
        }
    } else if let ("graph", Some(cmd_args)) = matches.subcommand() {
        let text_id = cmd_args.value_of("id").unwrap();
        if let Some(config) = config {
            let (build, project) = config.open()?;
            let rid = {
                if let Ok(resource_id) = ResourceId::from_str(text_id) {
                    build
                        .lookup_pathid(resource_id)
                        .ok_or(format!("ResourceId '{}' not found", resource_id))?
                } else {
                    ResourcePathId::from_str(text_id)
                        .map_err(|_e| format!("Invalid ResourcePathId '{}'", text_id))?
                }
            };
            let output =
                build.print_build_graph(rid, |rid| pretty_name_from_pathid(rid, &project, &config));
            println!("{}", output);
        } else {
            return Err("Configuration not found. Run 'data-scrape configure' first.".to_string());
        }
    } else if let ("configure", Some(cmd_args)) = matches.subcommand() {
        let config_path = Config::default_path();
        let workspace_dir = Config::workspace_dir();

        let code_paths = cmd_args.values_of("code_path").map_or_else(
            || {
                vec![
                    workspace_dir.join("lib/"),
                    workspace_dir.join("client/"),
                    workspace_dir.join("test/"),
                ]
            },
            |args| args.into_iter().map(PathBuf::from).collect::<Vec<_>>(),
        );

        let buildindex = cmd_args
            .value_of("buildindex")
            .map_or(workspace_dir.join("test/sample-data/temp/"), PathBuf::from);

        let project = cmd_args.value_of("project").map_or(
            workspace_dir.join("test/sample-data/project.index"),
            PathBuf::from,
        );

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
            buildindex,
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
                let arg: syn::LitStr = a.parse_args().unwrap();
                let arg = arg.value();

                let ty = ResourceType::new(arg.as_bytes());

                return Some((arg, ty));
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

// Finds all #[resource="name"] attributes in a file and returns (name, hashed name) tuple.
fn all_declared_resources(source: &Path) -> Vec<(String, ResourceType)> {
    let src = std::fs::read_to_string(&source).expect("Read file");
    let ast = syn::parse_file(&src).expect("Unable to parse file");
    find_resource_attribs(&ast.items)
}

// Reads an asset file, and prints out its header information.
#[allow(unsafe_code)]
fn parse_asset_file(path: impl AsRef<Path>) {
    let path = path.as_ref();
    let mut f = File::open(path).expect("unable to open asset file");

    let file_name = path.file_name().unwrap().to_string_lossy();
    let file_guid = u128::from_str_radix(&file_name, 16);
    if let Err(_e) = file_guid {
        // not an asset file, just ignore it
        return;
    }
    let file_guid = file_guid.unwrap();
    println!("\nasset {:032x}", file_guid);

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
            let asset_ref = unsafe {
                std::mem::transmute::<u128, ResourceId>(
                    f.read_u128::<LittleEndian>().expect("valid data"),
                )
            };
            println!("\t\treference: {}", asset_ref);
        }
    }

    let asset_type = unsafe {
        std::mem::transmute::<u32, ResourceType>(f.read_u32::<LittleEndian>().expect("valid data"))
    };
    println!("\tasset type: {}", asset_type);

    let asset_count = f.read_u64::<LittleEndian>().expect("valid data");
    println!("\tasset count: {}", asset_count);

    let nbytes = f.read_u64::<LittleEndian>().expect("valid data");
    println!("\tasset content size: {}", nbytes);
}

fn pretty_name_from_pathid(rid: &ResourcePathId, project: &Project, config: &Config) -> String {
    let mut output_text = String::new();

    if let Ok(source_name) = project.resource_name(rid.source_resource()) {
        output_text.push_str(&source_name.to_string());
    } else {
        output_text.push_str(&rid.source_resource().to_string());
    }

    let source_ty_pretty = config
        .type_map
        .get(&rid.source_resource().ty())
        .cloned()
        .unwrap_or_else(|| rid.source_resource().ty().to_string());
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
