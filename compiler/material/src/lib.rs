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

use legion_data_compiler::{
    compiler_api::{
        CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    compiler_utils::{hash_code_and_data, path_id_to_binary},
};
use legion_data_offline::ResourcePathId;
use legion_data_runtime::Resource;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &(
        legion_graphics_offline::Material::TYPE,
        legion_graphics_runtime::Material::TYPE,
    ),
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context
        .take_registry()
        .add_loader::<legion_graphics_offline::Material>()
        .create();

    let resource =
        resources.load_sync::<legion_graphics_offline::Material>(context.source.resource_id());

    let resource = resource.get(&resources).unwrap();

    let compiled_asset = {
        let mut c: Vec<u8> = vec![];
        c.append(&mut path_id_to_binary(&resource.albedo));
        c.append(&mut path_id_to_binary(&resource.normal));
        c.append(&mut path_id_to_binary(&resource.roughness));
        c.append(&mut path_id_to_binary(&resource.metalness));
        c
    };

    let asset = context.store(&compiled_asset, context.target_unnamed.clone())?;

    let mut resource_references: Vec<(ResourcePathId, ResourcePathId)> = Vec::new();

    let mut add_reference = |reference: &Option<ResourcePathId>| {
        if let Some(reference) = reference {
            resource_references.push((context.target_unnamed.clone(), reference.clone()));
        }
    };

    add_reference(&resource.albedo);
    add_reference(&resource.normal);
    add_reference(&resource.roughness);
    add_reference(&resource.metalness);

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references,
    })
}
