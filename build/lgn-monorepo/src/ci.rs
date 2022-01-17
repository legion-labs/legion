use std::process::{Command, Stdio};

use clap::Subcommand;
use lgn_tracing::{span_fn, span_scope};

use crate::cargo::{BuildArgs, SelectedPackageArgs};
use crate::{action_step, bench, build, check, clippy, fmt, lint, test, Error};
use crate::{context::Context, Result};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run all CI checks
    #[clap(name = "check")]
    Check {
        /// Run formatting check
        #[clap(long)]
        fmt: bool,
        #[clap(long)]
        gfx_no_api: bool,
        #[clap(long)]
        clippy: bool,
        #[clap(long)]
        repo_lints: bool,
        #[clap(long)]
        cargo_deny: bool,
    },
    #[clap(name = "test")]
    Test {
        /// Run build tests
        #[clap(long)]
        build: bool,
        /// Run run tests
        #[clap(long)]
        run: bool,
    },
    #[clap(name = "bench")]
    Bench {
        /// Run build tests
        #[clap(long)]
        build: bool,
        /// Run run tests
        #[clap(long)]
        run: bool,
    },
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    if let Some(ref command) = args.command {
        run_command(ctx, command)?;
    } else {
        let command = Commands::Check {
            fmt: true,
            gfx_no_api: true,
            clippy: true,
            repo_lints: true,
            cargo_deny: true,
        };
        run_command(ctx, &command)?;
        let command = Commands::Test {
            build: true,
            run: true,
        };
        run_command(ctx, &command)?;
        // do not run benches by default
        //let command = Commands::Bench {
        //    build: true,
        //    run: true,
        //};
        //run_command(ctx, &command)?;
    }
    Ok(())
}

fn run_command(ctx: &Context, command: &Commands) -> Result<()> {
    match command {
        Commands::Check {
            fmt,
            gfx_no_api,
            clippy,
            repo_lints,
            cargo_deny,
        } => {
            let all = !fmt && !gfx_no_api && !clippy && !repo_lints && !cargo_deny;
            if all || *fmt {
                check_fmt(ctx)?;
            }
            if all || *gfx_no_api {
                check_graphic_crate(ctx)?;
            }
            if all || *clippy {
                check_clippy(ctx)?;
            }
            if all || *repo_lints {
                check_repo_lints(ctx)?;
            }
            if all || *cargo_deny {
                check_cargo_deny(ctx)?;
            }
            Ok(())
        }
        Commands::Test { build, run } => {
            let all = !build && !run;
            if all || *build {
                test_build(ctx)?;
            }
            if all || *run {
                test_run(ctx)?;
            }
            Ok(())
        }
        Commands::Bench { build, run } => {
            let all = !build && !run;
            if all || *build {
                bench_build(ctx)?;
            }
            if all || *run {
                bench_run(ctx)?;
            }
            Ok(())
        }
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
fn check_repo_lints(ctx: &Context) -> Result<()> {
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

#[span_fn]
fn bench_build(ctx: &Context) -> Result<()> {
    action_step!("-- CI --", "Building benches");
    let args = bench::Args {
        package_args: SelectedPackageArgs {
            ..SelectedPackageArgs::default()
        },
        ..bench::Args::default()
    };
    bench::run(args, ctx)
}

#[span_fn]
fn bench_run(ctx: &Context) -> Result<()> {
    action_step!("-- CI --", "Running benches");
    let args = bench::Args {
        package_args: SelectedPackageArgs {
            ..SelectedPackageArgs::default()
        },
        ..bench::Args::default()
    };
    bench::run(args, ctx)
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
