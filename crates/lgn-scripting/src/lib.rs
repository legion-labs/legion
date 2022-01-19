//! Scripting library - currently using the MUN language

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

#[path = "../codegen/offline/mod.rs"]
#[cfg(feature = "offline")]
pub mod offline;

#[path = "../codegen/runtime/mod.rs"]
#[cfg(feature = "runtime")]
pub mod runtime;

pub mod components;

/*use lgn_ecs::prelude::*;

#[derive(Component)]
pub struct ScriptECSComponent {
    pub back_ref: ResourceId,
}*/

/*#[derive(Serialize, Deserialize)]
pub struct Script {

#[typetag::serde]
impl Component for Script {
    fn extract_build_deps(&self) -> Vec<ResourcePathId> {
        vec![]
    }
}*/

use std::{cell::RefCell, rc::Rc};

use components::ECSScriptComponent;
use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use mun_runtime::{invoke_fn, Runtime, RuntimeBuilder};
use std::str::FromStr;

#[derive(Default)]
struct RuntimeScripts {
    pub runtimes: Vec<(ECSScriptComponent, Rc<RefCell<Runtime>>)>,
}

#[derive(Default)]
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<RuntimeScripts>()
            .add_system(Self::tick_scripts);
    }
}

impl ScriptingPlugin {
    #[allow(clippy::needless_pass_by_value)]
    fn tick_scripts(
        mut runtimes: NonSendMut<'_, RuntimeScripts>,
        scripts: Query<'_, '_, &mut ECSScriptComponent>,
    ) {
        if runtimes.runtimes.is_empty() {
            for script in scripts.iter() {
                let runtime = RuntimeBuilder::new(&script.lib_path)
                    .spawn()
                    .expect("Failed to spawn Runtime");
                runtimes.runtimes.push((script.clone(), runtime));
            }
        }
        for runtime in &runtimes.runtimes {
            {
                let runtime_ref = runtime.1.borrow();
                let result: i64 = invoke_fn!(
                    runtime_ref,
                    &runtime.0.entry_fn,
                    i64::from_str(&runtime.0.input_values[0]).unwrap()
                )
                .unwrap();
                println!("fibonacci({}) = {}", &runtime.0.input_values[0], result);
            }

            // reload the script of the path changed
            runtime.1.borrow_mut().update();
        }
    }
}
