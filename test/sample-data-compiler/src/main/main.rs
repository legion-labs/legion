//! The runtime server is the portion of the Legion Engine that runs off runtime
//! data to simulate a world. It is tied to the lifetime of a runtime client.
//!
//! * Tracking Issue: [legion/crate/#xx](https://github.com/legion-labs/legion/issues/xx)
//! * Design Doc: [legion/book/project-resources](/book/todo.html)
//!

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

use clap::{App, Arg};
use lgn_data_offline::resource::ResourcePathName;
use sample_data_compiler::{offline_compiler, raw_loader};

fn main() {
    const ARG_PROJECT_DIR: &str = "root";
    const ARG_RESOURCE_NAME: &str = "resource";

    let args = App::new("Sample data compiler")
        .version(clap::crate_version!())
        .about("Will load RON files containing sample data, and generate offline resources and runtime assets, along with manifests.")
        .arg(Arg::with_name(ARG_PROJECT_DIR)
            .long(ARG_PROJECT_DIR)
            .takes_value(true)
            .help("Folder containing raw/ directory"))
        .arg(Arg::with_name(ARG_RESOURCE_NAME)
                .long(ARG_RESOURCE_NAME)
                .takes_value(true)
                .help("Path name of the resource to compile"))
        .get_matches();

    let project_dir = args.value_of(ARG_PROJECT_DIR).unwrap_or("test/sample-data");
    let root_resource = args
        .value_of(ARG_RESOURCE_NAME)
        .unwrap_or("/world/sample_1.ent");

    // generate contents of offline folder, from raw RON content
    raw_loader::build_offline(project_dir);

    // compile offline resources to runtime assets
    offline_compiler::build(project_dir, &ResourcePathName::from(root_resource));
}
