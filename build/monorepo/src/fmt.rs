// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lgn_tracing::span_fn;

use crate::{cargo::Cargo, context::Context, Error, Result};
use std::{ffi::OsString, process::Command};

#[derive(Debug, clap::Args, Default)]
pub struct Args {
    #[clap(long)]
    /// Run in 'check' mode. Exits with 0 if input is
    /// formatted correctly. Exits with 1 and prints a diff if
    /// formatting is required.
    pub(crate) check: bool,

    #[clap(long)]
    /// Run check on all packages in the workspace
    pub(crate) workspace: bool,

    #[clap(name = "ARGS", parse(from_os_str), last = true)]
    /// Pass through args to rustfmt
    pub(crate) args: Vec<OsString>,
}

#[span_fn]
pub fn run(args: Args, ctx: &Context) -> Result<()> {
    let mut pass_through_args = vec![];

    if args.check {
        pass_through_args.push("--check".into());
    }

    pass_through_args.extend(args.args);

    let mut cmd = Cargo::new(ctx, "fmt", true);

    if args.workspace {
        // cargo fmt doesn't have a --workspace flag, instead it uses the
        // old --all flag
        cmd.all();
    }

    cmd.pass_through(&pass_through_args).run()?;

    proto_fmt(ctx, args.check)
}

#[span_fn]
fn proto_fmt(ctx: &Context, check: bool) -> Result<()> {
    // walk all files to find .proto files
    let proto_files: Vec<_> = walkdir::WalkDir::new(ctx.workspace_root())
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            // ideally we would parse the .gitignore file and use it to filter
            let path = e.path().as_os_str().to_string_lossy().replace("\\", "/");
            if path.contains("/node_modules/") || path.contains("/target/") {
                return false;
            }
            path.rsplit('.')
                .next()
                .map(|ext| ext.eq_ignore_ascii_case("proto"))
                == Some(true)
        })
        .map(|e| e.path().to_owned())
        .collect();

    if check {
        // in check mode we will run clang-format on individual files
        // even if it costs more time, we will re-evaluate if it becomes an issue
        let mut explanations = String::new();
        for file in proto_files {
            let mut cmd = Command::new("clang-format");
            cmd.arg("-output-replacements-xml");
            cmd.arg(&file);
            let output = cmd
                .output()
                .map_err(|e| Error::new("clang-format failed").with_source(e))?;
            if output.status.success() {
                // clang-format lists replacements in XML format by opening a replacement tag.
                // example: <replacement offset='1182' length='7'>&#10;</replacement>
                if String::from_utf8_lossy(&output.stdout).contains("<replacement ") {
                    explanations.push_str(&format!(
                        "found replacements in {}\n",
                        file.to_string_lossy()
                    ));
                }
            } else {
                return Err(Error::new("clang-format failed").with_exit_code(output.status.code()));
            }
        }
        if explanations.is_empty() {
            Ok(())
        } else {
            Err(
                Error::new("proto files formatting check failed, run `cargo m fmt` to fix")
                    .with_explanation(explanations),
            )
        }
    } else {
        let mut cmd = Command::new("clang-format");
        cmd.arg("-i");
        cmd.args(&proto_files);
        let exist_status = cmd
            .status()
            .map_err(|e| Error::new("clang-format failed").with_source(e))?;
        if exist_status.success() {
            Ok(())
        } else {
            Err(Error::new("clang-format failed").with_exit_code(exist_status.code()))
        }
    }
}
