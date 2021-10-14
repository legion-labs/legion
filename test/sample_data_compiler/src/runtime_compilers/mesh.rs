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

mod offline_to_runtime;

use legion_data_compiler::{
    compiler_api::{
        compiler_main, CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError,
        DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use legion_data_offline::ResourcePathId;
use legion_data_runtime::Resource;
use offline_to_runtime::FromOffline;
use sample_data_compiler::{offline_data, runtime_data};
use std::env;

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &(offline_data::Mesh::TYPE, runtime_data::Mesh::TYPE),
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let mut resources = context
        .take_registry()
        .add_loader::<offline_data::Mesh>()
        .create();

    let mesh = resources.load_sync::<offline_data::Mesh>(context.source.content_id());
    let mesh = mesh.get(&resources).unwrap();

    let runtime_mesh = runtime_data::Mesh::from_offline(&mesh);
    let compiled_asset = bincode::serialize(&runtime_mesh).unwrap();

    let asset = context.store(&compiled_asset, context.target_unnamed.clone())?;

    let mut resource_references: Vec<(ResourcePathId, ResourcePathId)> = Vec::new();
    for sub_mesh in &mesh.sub_meshes {
        if let Some(material) = &sub_mesh.material {
            resource_references.push((context.target_unnamed.clone(), material.clone()));
        }
    }

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references,
    })
}

fn main() -> Result<(), CompilerError> {
    compiler_main(env::args(), &COMPILER_INFO)
}
