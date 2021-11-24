//! Generic data codegen test (offline)

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
#![allow(clippy::needless_update)]

mod debug_cube {
    include!(concat!(env!("OUT_DIR"), "/debug_cube.rs"));
}
mod entity_dc {
    include!(concat!(env!("OUT_DIR"), "/entity_dc.rs"));
}
mod instance_dc {
    include!(concat!(env!("OUT_DIR"), "/instance_dc.rs"));
}
mod test_entity {
    include!(concat!(env!("OUT_DIR"), "/test_entity.rs"));
}
mod light_component {
    include!(concat!(env!("OUT_DIR"), "/light_component.rs"));
}
mod static_mesh_component {
    include!(concat!(env!("OUT_DIR"), "/static_mesh_component.rs"));
}

mod transform_component {
    include!(concat!(env!("OUT_DIR"), "/transform_component.rs"));
}

mod rotation_component {
    include!(concat!(env!("OUT_DIR"), "/rotation_component.rs"));
}

pub use debug_cube::*;
pub use entity_dc::*;
pub use instance_dc::*;
pub use test_entity::*;

pub use light_component::*;
pub use rotation_component::*;
pub use static_mesh_component::*;
pub use transform_component::*;

pub fn register_resource_types(
    registry: lgn_data_offline::resource::ResourceRegistryOptions,
) -> lgn_data_offline::resource::ResourceRegistryOptions {
    let registry = debug_cube::register_resource_types(registry);
    let registry = entity_dc::register_resource_types(registry);
    let registry = instance_dc::register_resource_types(registry);
    test_entity::register_resource_types(registry)
}
