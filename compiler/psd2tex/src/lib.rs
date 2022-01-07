// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this
// section
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

use lgn_data_compiler::{
    compiler_api::{
        CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::Transform;
use lgn_data_runtime::Resource;
use lgn_graphics_offline::PsdFile;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_offline::PsdFile::TYPE,
        lgn_graphics_offline::Texture::TYPE,
    ),
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context
        .take_registry()
        .add_loader::<lgn_graphics_offline::PsdFile>()
        .create();

    let resource =
        resources.load_sync::<lgn_graphics_offline::PsdFile>(context.source.resource_id());

    let resource = resource.get(&resources).unwrap();

    let mut compiled_resources = vec![];

    let compiled_content = {
        let final_image = resource
            .final_texture()
            .ok_or(CompilerError::CompilationError(
                "Failed to generate texture",
            ))?;
        serde_json::to_vec(&final_image)
            .map_err(|_e| CompilerError::CompilationError("Failed to serialize"))?
    };

    let output = context.store(&compiled_content, context.target_unnamed.clone())?;
    compiled_resources.push(output);

    let compile_layer = |psd: &PsdFile, layer_name| -> Result<Vec<u8>, CompilerError> {
        let image = psd.layer_texture(layer_name).unwrap();
        serde_json::to_vec(&image)
            .map_err(|_e| CompilerError::CompilationError("Failed to serialize"))
    };

    for layer_name in resource
        .layer_list()
        .ok_or(CompilerError::CompilationError(
            "Failed to extract layer names",
        ))?
    {
        let pixels = compile_layer(&resource, layer_name)?;
        let output = context.store(&pixels, context.target_unnamed.new_named(layer_name))?;
        compiled_resources.push(output);
    }

    Ok(CompilationOutput {
        compiled_resources,
        resource_references: vec![],
    })
}
