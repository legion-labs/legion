// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    action_step,
    cargo::{BuildArgs, CargoCommand, SelectedPackageArgs},
    context::Context,
    Error, Result,
};
use lgn_tracing::{info, trace_function};
use std::{
    ffi::OsString,
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(flatten)]
    pub(crate) package_args: SelectedPackageArgs,
    #[clap(long, short)]
    /// Skip running expensive diem testsuite integration tests
    unit: bool,
    #[clap(long)]
    /// Only run doctests
    doc: bool,
    #[clap(flatten)]
    pub(crate) build_args: BuildArgs,
    #[clap(long)]
    /// Do not fast fail the run if tests (or test executables) fail
    no_fail_fast: bool,
    #[clap(long)]
    /// Do not run tests, only compile the test executables
    no_run: bool,
    #[clap(long, parse(from_os_str))]
    /// Directory to output HTML coverage report (using grcov)
    html_cov_dir: Option<PathBuf>,
    #[clap(name = "TESTNAME", parse(from_os_str))]
    testname: Option<OsString>,
    #[clap(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

#[trace_function]
pub fn run(mut args: Args, ctx: &Context) -> Result<()> {
    let packages = args.package_args.to_selected_packages(ctx)?;
    //if args.unit {
    //    packages.add_excludes(ctx.config().system_tests().iter().map(|(p, _)| p.as_str()));
    //}

    args.args.extend(args.testname.clone());

    let llvm_profile_key = "LLVM_PROFILE_FILE";
    let llvm_profile_path: &str = "target/debug/xtest-%p-%m.profraw";
    let llvm_profile_path_ignored = "target/debug/ignored-%p-%m.profraw";

    let generate_coverage = args.html_cov_dir.is_some();
    let env_vars = if generate_coverage {
        if !ctx.installer().install_via_cargo_if_needed(ctx, "grcov") {
            return Err(Error::new("Could not install grcov"));
        }

        let shared_envionment = vec![
            ("RUSTC_BOOTSTRAP", Some("1")),
            // Recommend flags for use with grcov, with these flags removed: -Copt-level=0, -Clink-dead-code.
            // for more info see:  https://github.com/mozilla/grcov#example-how-to-generate-gcda-fiels-for-a-rust-project
            ("RUSTFLAGS", Some("-Zinstrument-coverage")),
            ("RUST_MIN_STACK", Some("8388608")),
        ];

        let mut build_env_vars = shared_envionment.clone();
        build_env_vars.push((llvm_profile_key, Some(llvm_profile_path_ignored)));

        action_step!(
            "Coverage",
            "Performing a seperate \"cargo build\" before running tests and collecting coverage"
        );

        let mut direct_args = Vec::new();
        args.build_args.add_args(&mut direct_args);

        let build = CargoCommand::Build {
            direct_args: direct_args.as_slice(),
            args: &[],
            env: build_env_vars.as_slice(),
            skip_sccache: true,
        };
        let build_result = build.run_on_packages(ctx, &packages);

        if !args.no_fail_fast && build_result.is_err() {
            return build_result;
        }

        let mut output = shared_envionment.clone();
        output.push((llvm_profile_key, Some(llvm_profile_path)));
        output
    } else {
        vec![]
    };

    let mut direct_args = Vec::new();
    args.build_args.add_args(&mut direct_args);
    if args.no_run {
        direct_args.push(OsString::from("--no-run"));
    };
    if args.no_fail_fast {
        direct_args.push(OsString::from("--no-fail-fast"));
    };
    if args.doc {
        direct_args.push(OsString::from("--doc"));
    }

    let cmd = CargoCommand::Test {
        direct_args: direct_args.as_slice(),
        args: &args.args,
        env: &env_vars,
        skip_sccache: generate_coverage,
    };

    let cmd_result = cmd.run_on_packages(ctx, &packages);

    if !args.no_fail_fast && cmd_result.is_err() {
        return cmd_result;
    }

    if let Some(html_cov_dir) = &args.html_cov_dir {
        action_step!("Coverage", "Generating HTML coverage report");
        create_dir_all(&html_cov_dir).map_err(|err| {
            Error::new(format!(
                "Failed to create grcov directory {}",
                html_cov_dir.display()
            ))
            .with_source(err)
        })?;
        let html_cov_path = &html_cov_dir.canonicalize().map_err(|err| {
            Error::new(format!(
                "Failed to create canonicalize directory {}",
                html_cov_dir.display()
            ))
            .with_source(err)
        })?;
        info!("created {}", &html_cov_path.to_string_lossy());
        exec_grcov(ctx, html_cov_path, llvm_profile_path)?;
    }
    cmd_result
}

fn exec_grcov(ctx: &Context, html_cov_path: &Path, llvm_profile_path: &str) -> Result<()> {
    let debug_dir = ctx.workspace_root().join("target/debug/");
    let mut grcov_html = Command::new("grcov");
    //grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing --ignore "/*" -o $HOME/output/
    grcov_html
        .current_dir(ctx.workspace_root())
        //output file from coverage: gcda files
        .arg(ctx.workspace_root().as_os_str())
        .arg("--binary-path")
        .arg(debug_dir.as_os_str())
        //source code location
        .arg("-s")
        .arg(ctx.workspace_root().as_os_str())
        //html output
        .arg("-t")
        .arg("html")
        .arg("--branch")
        .arg("--ignore")
        .arg("/*")
        .arg("--ignore")
        .arg("x/*")
        .arg("--ignore")
        .arg("testsuite/*")
        .arg("--ignore-not-existing")
        .arg("-o")
        .arg(html_cov_path);
    info!("Build grcov Html Coverage Report");
    info!("{:?}", grcov_html);
    grcov_html.env("LLVM_PROFILE_FILE", llvm_profile_path);
    grcov_html.env("RUSTFLAGS", "-Zinstrument-coverage");
    grcov_html.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    if let Some(err) = grcov_html.output().err() {
        Err(Error::new("Failed to generate html output with grcov").with_source(err))
    } else {
        Ok(())
    }
}
