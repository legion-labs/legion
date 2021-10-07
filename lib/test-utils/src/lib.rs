//! test-utils : provides utility functions to help you build integration & unit tests

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

use std::process::Command;
use std::{
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

//std::fs::remove_dir_all leaves read-only files and reports an error
fn force_delete_all(dir: &Path) {
    fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                }
                cb(&entry);
            }
        }
        Ok(())
    }

    visit_dirs(dir, &|entry| {
        let p = entry.path();
        let meta = entry.metadata().unwrap();
        if meta.is_dir() {
            fs::remove_dir(p).unwrap();
        } else {
            let mut perm = meta.permissions();
            if perm.readonly() {
                perm.set_readonly(false);
                fs::set_permissions(&p, perm).unwrap();
            }
            fs::remove_file(&p).unwrap();
        }
    })
    .unwrap();
}

// create_test_dir creates a directory (or cleans its contents) under the `target` folder that will outlive the execution of the test.
pub fn create_test_dir(parent_path: &Path, test_name: &str) -> PathBuf {
    let path = parent_path.join(test_name);

    if path.exists() {
        force_delete_all(&path);
    }
    std::fs::create_dir_all(&path).unwrap();
    path
}

// syscall will execute `command` from the `wd` directory and validate that the error code matches `should_succeed`
pub fn syscall(command: &str, wd: &Path, args: &[&str], should_succeed: bool) {
    println!("{} {}", command, args.join(" "));
    let status = Command::new(command)
        .current_dir(wd)
        .args(args)
        .status()
        .expect("failed to execute command");
    assert_eq!(status.success(), should_succeed);
}
