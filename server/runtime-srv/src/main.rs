//! The runtime server is the portion of the Legion Engine that runs off runtime
//! data to simulate a world. It is tied to the lifetime of a runtime client.
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)
//!

// BEGIN - Legion Labs lints v0.5
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
// END - Legion Labs standard lints v0.5
// crate-specific exceptions:
#![allow()]

use clap::Arg;
use legion_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use legion_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use legion_core::CorePlugin;
use legion_transform::prelude::*;
use legion_utils::Duration;

fn main() {
    const ARG_NAME_CAS: &str = "cas";
    const ARG_NAME_MANIFEST: &str = "manifest";
    const ARG_NAME_ROOT: &str = "root";

    let args = clap::App::new("Legion Labs runtime engine")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .about("Server that will run with runtime data, and execute world simulation, ready to be streamed to a runtime client.")
        .arg(Arg::with_name(ARG_NAME_CAS)
            .long(ARG_NAME_CAS)
            .takes_value(true)
            .help("Path to folder containing the content storage files"))
        .arg(Arg::with_name(ARG_NAME_MANIFEST)
            .long(ARG_NAME_MANIFEST)
            .takes_value(true)
            .help("Path to the game manifest"))
        .arg(Arg::with_name(ARG_NAME_ROOT)
            .long(ARG_NAME_ROOT)
            .takes_value(true)
            .help("Root object to load, usually a world"))
        .get_matches();

    let content_store_addr = args
        .value_of(ARG_NAME_CAS)
        .unwrap_or("test/sample_data/temp");

    let game_manifest = args
        .value_of(ARG_NAME_MANIFEST)
        .unwrap_or("test/sample_data/runtime/game.manifest");

    // default root object is in sample data
    // /world/sample_1.ent
    // resource-id: d004cd1c00000000445b0fc7dae84c6e
    // asset-id: 019c822300000000c340a8a5eba2f2bb
    // checksum: 0000000000000000f836a7f2fd844acf

    let root_asset = args
        .value_of(ARG_NAME_ROOT)
        .unwrap_or("019c822300000000c340a8a5eba2f2bb");

    // Start app with 60 fps
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(CorePlugin::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(TransformPlugin::default())
        .insert_resource(AssetRegistrySettings::new(
            content_store_addr,
            game_manifest,
            Some(root_asset),
        ))
        .add_plugin(AssetRegistryPlugin::default())
        .run();
}
