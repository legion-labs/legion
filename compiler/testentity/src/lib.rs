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

// TODO: Replace by using Reflection conversion
//include!(concat!(env!("OUT_DIR"), "/compiler_testentity.rs"));

use lgn_data_compiler::{
    compiler_api::{
        CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::{ResourcePathId, Transform};
use lgn_data_runtime::Resource;
use std::env;
type OfflineType = generic_data::offline::TestEntity;
type RuntimeType = generic_data::runtime::TestEntity;
pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "5274493235039250438",
    transform: &Transform::new(OfflineType::TYPE, RuntimeType::TYPE),
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};
fn extract_resource_dependencies(_offline: &OfflineType) -> Option<Vec<ResourcePathId>> {
    None
}

fn compile_resource(offline: &OfflineType) -> RuntimeType {
    RuntimeType {
        test_string: offline.test_string.clone(),
        test_color: offline.test_color,
        test_position: offline.test_position,
        test_rotation: offline.test_rotation,
        test_bool: offline.test_bool,
        test_float32: offline.test_float32,
        test_int: offline.test_int,
        test_blob: offline.test_blob.clone(),
        test_sub_type: generic_data::runtime::TestSubType1::default(),
        test_option_set: None,
        test_option_none: None,
    }
}
fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context.take_registry().add_loader::<OfflineType>().create();
    let offline_resource = resources.load_sync::<OfflineType>(context.source.resource_id());
    let offline_resource = offline_resource.get(&resources).unwrap();
    let runtime_resource = compile_resource(&offline_resource);
    let compiled_asset = bincode::serialize(&runtime_resource).unwrap();
    let resource_references = extract_resource_dependencies(&offline_resource);
    let resource_references: Vec<(ResourcePathId, ResourcePathId)> = resource_references
        .unwrap_or_default()
        .into_iter()
        .map(|res| (context.target_unnamed.clone(), res))
        .collect();
    let asset = context.store(&compiled_asset, context.target_unnamed.clone())?;
    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references,
    })
}
