use std::process::{Command, Stdio};

use lgn_tracing::{span_fn, span_scope};

use crate::cargo::{BuildArgs, SelectedPackageArgs};
use crate::{action_step, build, check, clippy, fmt, lint, skip_step, test, Error};
use crate::{context::Context, Result};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Run CI checks
    #[clap(
        long,
        multiple_values = true,
        use_delimiter = true,
        possible_values = &["all", "fmt", "gfx-no-api", "clippy", "mlint", "cargo-deny"],
    )]
    checks: Vec<String>,
    /// Run CI tests
    #[clap(
        long,
        multiple_values = true,
        use_delimiter = true,
        possible_values = &["all", "build", "run"],
    )]
    tests: Vec<String>,
}

type CheckFn = fn(&Context) -> Result<()>;

#[span_fn]
pub fn run(mut args: Args, ctx: &Context) -> Result<()> {
    if args.checks.is_empty() && args.tests.is_empty() {
        args.checks.push("all".into());
        args.tests.push("all".into());
    }
    if !args.checks.is_empty() {
        action_step!("-- CI --", "Running checks");
        for (check_name, check_fn) in &[
            ("fmt", check_fmt as CheckFn),
            ("gfx-no-api", check_graphic_crate as CheckFn),
            ("clippy", check_clippy as CheckFn),
            ("mlint", check_monorepo_lints as CheckFn),
            ("cargo-deny", check_cargo_deny as CheckFn),
        ] {
            run_step(ctx, "checks", check_name, *check_fn, &args.checks)?;
        }
    } else {
        skip_step!("-- CI --", "Skipping checks");
    }

    if !args.tests.is_empty() {
        action_step!("-- CI --", "Running tests");
        for (check_name, check_fn) in &[
            ("build", test_build as CheckFn),
            ("run", test_run as CheckFn),
        ] {
            run_step(ctx, "tests", check_name, *check_fn, &args.tests)?;
        }
    } else {
        skip_step!("-- CI --", "Skipping tests");
    }
    Ok(())
}

fn run_step(
    ctx: &Context,
    group: &str,
    name: &str,
    check_fn: fn(&Context) -> Result<()>,
    args: &[String],
) -> Result<()> {
    if args
        .iter()
        .map(String::as_str)
        .any(|check| check == "all" || check == name)
    {
        check_fn(ctx).map_err(|e| {
            Error::new(format!(
                "failed to run {}, to re-run: `cargo ci --{}={}`",
                name, group, name
            ))
            .with_exit_code(e.exit_code())
            .with_source(e)
        })
    } else {
        skip_step!("-- CI --", "Skipping step {} from {}", name, group);
        Ok(())
    }
}

#[span_fn]
fn check_clippy(ctx: &Context) -> Result<()> {
    action_step!("-- CI --", "Running clippy checks");
    let args = clippy::Args {
        package_args: SelectedPackageArgs {
            workspace: true,
            ..SelectedPackageArgs::default()
        },
        build_args: BuildArgs {
            all_targets: true,
            all_features: true,
            locked: true,
            ..BuildArgs::default()
        },
        args: vec!["-D".into(), "warnings".into()],
        ..clippy::Args::default()
    };
    clippy::run(&args, ctx)
}

#[span_fn]
fn check_fmt(ctx: &Context) -> Result<()> {
    action_step!("-- CI --", "Running formatting checks");
    span_scope!("fmt");
    let args = fmt::Args {
        check: true,
        workspace: true,
        ..fmt::Args::default()
    };
    fmt::run(args, ctx)
}

#[span_fn]
fn check_graphic_crate(ctx: &Context) -> Result<()> {
    action_step!("-- CI --", "Running check on the graphics crate");
    let args = check::Args {
        package_args: SelectedPackageArgs {
            package: vec!["lgn-graphics-api".into()],
            ..SelectedPackageArgs::default()
        },
        build_args: BuildArgs {
            all_targets: true,
            locked: true,
            ..BuildArgs::default()
        },
    };
    check::run(&args, ctx)
}

#[span_fn]
fn check_monorepo_lints(ctx: &Context) -> Result<()> {
    action_step!("-- CI --", "Running monorepo lints");
    lint::run(&lint::Args::default(), ctx)
}

#[span_fn]
fn check_cargo_deny(ctx: &Context) -> Result<()> {
    action_step!("-- CI --", "Running cargo deny lints");
    if !ctx
        .installer()
        .install_via_cargo_if_needed(ctx, "cargo-deny")
    {
        return Err(Error::new("could not find/install cargo-deny"));
    }
    let mut cargo_deny = Command::new("cargo");
    cargo_deny
        .current_dir(ctx.workspace_root())
        //output file from coverage: gcda files
        .arg("deny")
        .arg("check");

    cargo_deny.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let output = cargo_deny.output().map_err(Error::from_source)?;
    if !output.status.success() {
        return Err(
            Error::new("failed to run `cargo deny check`").with_exit_code(output.status.code())
        );
    }
    Ok(())
}

#[span_fn]
fn test_build(ctx: &Context) -> Result<()> {
    action_step!("-- CI --", "Running tests build");
    {
        let args = build::Args {
            package_args: SelectedPackageArgs {
                package: vec!["lgn-compiler-*".into()],
                ..SelectedPackageArgs::default()
            },
            build_args: BuildArgs::default(),
            ..build::Args::default()
        };
        build::run(args, ctx)?;
    }
    let args = test::Args {
        package_args: SelectedPackageArgs {
            ..SelectedPackageArgs::default()
        },
        build_args: BuildArgs::default(),
        no_run: true,
        ..test::Args::default()
    };
    test::run(args, ctx)
}

#[span_fn]
fn test_run(ctx: &Context) -> Result<()> {
    action_step!("-- CI --", "Running tests");
    let args = test::Args {
        package_args: SelectedPackageArgs {
            ..SelectedPackageArgs::default()
        },
        build_args: BuildArgs::default(),
        args: if machine_has_discreet_gpu()? {
            vec![]
        } else {
            action_step!("-- CI --", "Skipping Gpu tests");
            vec!["--skip".into(), "gpu_".into()]
        },
        ..test::Args::default()
    };
    test::run(args, ctx)
}

fn machine_has_discreet_gpu() -> Result<bool> {
    if cfg!(target_os = "windows") {
        let mut cmd = Command::new("wmic");
        cmd.args(&["path", "Win32_VideoController", "Get", "name"]);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let output = cmd
            .output()
            .map_err(|err| Error::new("Failed to run `wmic`").with_source(err))?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.contains("NVIDIA"))
        } else {
            Ok(false)
        }
    } else {
        let mut cmd = Command::new("lspci");
        cmd.arg("-nn");
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let output = cmd
            .output()
            .map_err(|err| Error::new("Failed to run `lspci`").with_source(err))?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.contains("NVIDIA"))
        } else {
            Ok(false)
        }
    }
}
