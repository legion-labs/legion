//! The runtime server is the portion of the Legion Engine that runs off runtime
//! data to simulate a world. It is tied to the lifetime of a runtime client.
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)
//!

// BEGIN - Legion Labs lints v0.2
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::use_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    broken_intra_doc_links,
    private_intra_doc_links,
    missing_crate_level_docs,
    rust_2018_idioms
)]
// END - Legion Labs standard lints v0.2
// crate-specific exceptions:
#![allow()]

mod asset_registry_plugin;

use clap::Arg;
use legion_app::{prelude::*, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use legion_core::CorePlugin;
use legion_ecs::prelude::*;
use legion_transform::prelude::*;
use legion_utils::Duration;

use crate::asset_registry_plugin::{AssetRegistryPlugin, AssetRegistrySettings};

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
    // resource-id: 5004cd1c00000000cc2e0e75fdc1bc61
    // asset-id: 819c8223000000002355395392dd415c
    // current runtime/cas checksum: 917911064617515641
    let root_asset = args
        .value_of(ARG_NAME_ROOT)
        .unwrap_or("819c8223000000002355395392dd415c");

    // Start app with 60 fps
    App::new()
        .insert_resource(AssetRegistrySettings::new(
            content_store_addr,
            game_manifest,
            root_asset,
        ))
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugin(CorePlugin::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(TransformPlugin::default())
        .add_plugin(AssetRegistryPlugin::default())
        .add_system(frame_counter)
        .run();
}

fn frame_counter(mut state: Local<'_, CounterState>) {
    if state.count % 60 == 0 {
        println!("{}", state.count / 60);
    }
    state.count += 1;
}

#[derive(Default)]
struct CounterState {
    count: u32,
}
