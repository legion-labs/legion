// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Instant;

use indexmap::IndexMap;
use lgn_tracing::{error, info, span_fn, warn};
use monorepo_base::sccache::{
    apply_sccache_if_possible, log_sccache_stats, sccache_should_run, stop_sccache_server,
};

use crate::context::Context;
use crate::{Error, Result};

mod build_args;
pub use build_args::*;

mod selected_packages;
pub use selected_packages::*;

const SECRET_ENVS: &[&str] = &["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"];

pub struct Cargo {
    inner: Command,
    pass_through_args: Vec<OsString>,
    env_additions: IndexMap<OsString, Option<OsString>>,
    on_close: fn(),
}

impl Drop for Cargo {
    fn drop(&mut self) {
        (self.on_close)();
    }
}

impl Cargo {
    pub fn new<S: AsRef<OsStr>>(ctx: &Context, command: S, skip_sccache: bool) -> Self {
        let mut inner = Command::new("cargo");
        //sccache apply
        let envs: IndexMap<OsString, Option<OsString>> = if !skip_sccache {
            let result = apply_sccache_if_possible(
                ctx.workspace_root(),
                ctx.installer(),
                &ctx.config().cargo.sccache,
            );
            match result {
                Ok(env) => env
                    .iter()
                    .map(|(key, option)| {
                        if let Some(val) = option {
                            (
                                OsString::from(key.to_owned()),
                                Some(OsString::from(val.clone())),
                            )
                        } else {
                            (OsString::from(key.to_owned()), None)
                        }
                    })
                    .collect(),
                Err(hmm) => {
                    warn!("Could not install sccache: {}", hmm);
                    IndexMap::new()
                }
            }
        } else {
            IndexMap::new()
        };
        let on_drop = if !skip_sccache
            && sccache_should_run(ctx.workspace_root(), &ctx.config().cargo.sccache, false)
        {
            || {
                log_sccache_stats();
                stop_sccache_server();
            }
        } else {
            || ()
        };
        inner.arg(command);
        Self {
            inner,
            pass_through_args: Vec::new(),
            env_additions: envs,
            on_close: on_drop,
        }
    }

    pub fn all(&mut self) -> &mut Self {
        self.inner.arg("--all");
        self
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    pub fn packages(&mut self, packages: &SelectedPackages<'_>) -> &mut Self {
        match &packages.includes {
            SelectedInclude::Workspace => {
                self.inner.arg("--workspace");
                for &e in &packages.excludes {
                    self.inner.args(&["--exclude", e]);
                }
            }
            SelectedInclude::Includes(includes) => {
                for &p in includes {
                    if !packages.excludes.contains(p) {
                        self.inner.args(&["--package", p]);
                    }
                }
            }
        }
        self
    }

    /// Adds a series of arguments to x's target command.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.inner.args(args);
        self
    }

    /// Adds an argument to x's target command.
    #[allow(dead_code)]
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    /// Adds "Pass Through" arguments to x's target command.
    /// Pass through arguments appear after a double dash " -- " and may
    /// not be handled/checked by x's target command itself, but an underlying executable.
    pub fn pass_through<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for a in args {
            self.pass_through_args.push(a.as_ref().to_owned());
        }
        self
    }

    /// Passes extra environment variables to x's target command.
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, Option<V>)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (key, val) in vars {
            self.env(key, val);
        }
        self
    }

    /// Passes an extra environment variable to x's target command.
    pub fn env<K, V>(&mut self, key: K, val: Option<V>) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        let converted_val = val.map(|s| s.as_ref().to_owned());

        self.env_additions
            .insert(key.as_ref().to_owned(), converted_val);
        self
    }

    pub fn run(&mut self) -> Result<()> {
        self.inner.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        self.do_run(cfg!(debug_assertions)).map(|_| ())
    }

    /// Runs this command, capturing the standard output into a `Vec<u8>`.
    /// Standard error is forwarded.
    #[allow(dead_code)]
    pub fn run_with_output(&mut self) -> Result<Vec<u8>> {
        self.inner.stderr(Stdio::inherit());
        self.do_run(cfg!(debug_assertions)).map(|o| o.stdout)
    }

    /// Internal run command, where the magic happens.
    /// If log is true, any environment variable overrides will be logged, the full command will be logged,
    /// and after the command's output reaches stdout, the command will be printed again along with the time took
    /// to process the command (wallclock) in ms.
    #[span_fn]
    fn do_run(&mut self, log: bool) -> Result<Output> {
        // these arguments are passed through cargo/x to underlying executable (test, clippy, etc)
        if !self.pass_through_args.is_empty() {
            self.inner.arg("--").args(&self.pass_through_args);
        }

        // once all the arguments are added to the command we can log it.
        if log {
            self.env_additions.iter().for_each(|(name, value_option)| {
                if let Some(env_val) = value_option {
                    if SECRET_ENVS.contains(&name.to_str().unwrap_or_default()) {
                        info!("export {:?}=********", name);
                    } else {
                        info!("export {:?}={:?}", name, env_val);
                    }
                } else {
                    info!("unset {:?}", name);
                }
            });
            info!("Executing: {:?}", &self.inner);
        }
        // process environment additions, removing Options that are none...
        for (key, option_value) in &self.env_additions {
            if let Some(value) = option_value {
                self.inner.env(key, value);
            } else {
                self.inner.env_remove(key);
            }
        }

        let now = Instant::now();
        let output = self.inner.output().map_err(Error::from_source)?;
        // once the command has been executed we log it's success or failure.
        if log {
            if output.status.success() {
                info!(
                    "Completed in {}ms: {:?}",
                    now.elapsed().as_millis(),
                    &self.inner
                );
            } else {
                error!(
                    "Failed in {}ms: {:?}",
                    now.elapsed().as_millis(),
                    &self.inner
                );
            }
        }
        if !output.status.success() {
            return Err(
                Error::new("failed to run cargo command").with_exit_code(output.status.code())
            );
        }
        Ok(output)
    }
}

pub enum CargoCommand<'a> {
    Bench {
        direct_args: &'a [OsString],
        args: &'a [OsString],
        env: &'a [(&'a str, Option<&'a str>)],
    },
    Check {
        direct_args: &'a [OsString],
    },
    Clippy {
        direct_args: &'a [OsString],
        args: &'a [OsString],
    },
    Doc {
        direct_args: &'a [OsString],
        args: &'a [OsString],
        env: &'a [(&'a str, Option<&'a str>)],
    },
    Fix {
        direct_args: &'a [OsString],
        args: &'a [OsString],
    },
    Test {
        direct_args: &'a [OsString],
        args: &'a [OsString],
        env: &'a [(&'a str, Option<&'a str>)],
        skip_sccache: bool,
    },
    Build {
        direct_args: &'a [OsString],
        args: &'a [OsString],
        env: &'a [(&'a str, Option<&'a str>)],
        skip_sccache: bool,
    },
    Run {
        direct_args: &'a [OsString],
        args: &'a [OsString],
        env: &'a [(&'a str, Option<&'a str>)],
    },
}

impl<'a> CargoCommand<'a> {
    pub fn skip_sccache(&self) -> bool {
        match self {
            CargoCommand::Build { skip_sccache, .. } | CargoCommand::Test { skip_sccache, .. } => {
                *skip_sccache
            }
            _ => false,
        }
    }

    pub fn run_on_packages(&self, ctx: &Context, packages: &SelectedPackages<'_>) -> Result<()> {
        // Early return if we have no packages to run.
        if !packages.should_invoke() {
            info!("no packages to {}: exiting early", self.as_str());
            return Ok(());
        }

        let mut cargo = self.prepare_cargo(ctx, packages);
        cargo.run()
    }

    /// Runs this command on the selected packages, returning the standard output as a bytestring.
    #[allow(dead_code)]
    pub fn run_capture_stdout(
        &self,
        ctx: &Context,
        packages: &SelectedPackages<'_>,
    ) -> Result<Vec<u8>> {
        // Early return if we have no packages to run.
        if !packages.should_invoke() {
            info!("no packages to {}: exiting early", self.as_str());
            Ok(vec![])
        } else {
            let mut cargo = self.prepare_cargo(ctx, packages);
            cargo.args(&["--message-format", "json-render-diagnostics"]);
            Ok(cargo.run_with_output()?)
        }
    }

    fn prepare_cargo(&self, ctx: &Context, packages: &SelectedPackages<'_>) -> Cargo {
        let mut cargo = Cargo::new(ctx, self.as_str(), self.skip_sccache());
        cargo
            .current_dir(ctx.workspace_root())
            .args(self.direct_args())
            .packages(packages)
            .pass_through(self.pass_through_args())
            .envs(self.get_extra_env().to_owned());

        cargo
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CargoCommand::Bench { .. } => "bench",
            CargoCommand::Check { .. } => "check",
            CargoCommand::Clippy { .. } => "clippy",
            CargoCommand::Doc { .. } => "doc",
            CargoCommand::Fix { .. } => "fix",
            CargoCommand::Test { .. } => "test",
            CargoCommand::Build { .. } => "build",
            CargoCommand::Run { .. } => "run",
        }
    }

    fn pass_through_args(&self) -> &[OsString] {
        match self {
            CargoCommand::Bench { args, .. }
            | CargoCommand::Clippy { args, .. }
            | CargoCommand::Doc { args, .. }
            | CargoCommand::Fix { args, .. }
            | CargoCommand::Test { args, .. }
            | CargoCommand::Build { args, .. }
            | CargoCommand::Run { args, .. } => args,
            CargoCommand::Check { .. } => &[],
        }
    }

    fn direct_args(&self) -> &[OsString] {
        match self {
            CargoCommand::Bench { direct_args, .. }
            | CargoCommand::Check { direct_args, .. }
            | CargoCommand::Clippy { direct_args, .. }
            | CargoCommand::Doc { direct_args, .. }
            | CargoCommand::Fix { direct_args, .. }
            | CargoCommand::Test { direct_args, .. }
            | CargoCommand::Build { direct_args, .. }
            | CargoCommand::Run { direct_args, .. } => direct_args,
        }
    }

    pub fn get_extra_env(&self) -> &[(&str, Option<&str>)] {
        match self {
            CargoCommand::Bench { env, .. }
            | CargoCommand::Build { env, .. }
            | CargoCommand::Doc { env, .. }
            | CargoCommand::Test { env, .. }
            | CargoCommand::Run { env, .. } => env,
            CargoCommand::Check { .. } | CargoCommand::Clippy { .. } | CargoCommand::Fix { .. } => {
                &[]
            }
        }
    }
}
