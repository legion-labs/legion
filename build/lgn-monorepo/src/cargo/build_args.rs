// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::{ArgEnum, Args};
use std::ffi::OsString;

#[derive(ArgEnum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Coloring {
    Auto,
    Always,
    Never,
}

impl Default for Coloring {
    fn default() -> Self {
        Self::Auto
    }
}

/// Arguments for controlling cargo build and other similar commands (like check).
#[derive(Debug, Args, Default, Clone)]
pub struct BuildArgs {
    #[clap(long, short)]
    /// No output printed to stdout
    pub(crate) quiet: bool,
    #[clap(long, short)]
    /// Number of parallel build jobs, defaults to # of CPUs
    pub(crate) jobs: Option<u16>,
    #[clap(long)]
    /// Only this package's library
    pub(crate) lib: bool,
    #[clap(long, number_of_values = 1)]
    /// Only the specified binary
    pub(crate) bin: Vec<String>,
    #[clap(long)]
    /// All binaries
    pub(crate) bins: bool,
    #[clap(long, number_of_values = 1)]
    /// Only the specified example
    pub(crate) example: Vec<String>,
    #[clap(long)]
    /// All examples
    pub(crate) examples: bool,
    #[clap(long, number_of_values = 1)]
    /// Only the specified test target
    pub(crate) test: Vec<String>,
    #[clap(long)]
    /// All tests
    pub(crate) tests: bool,
    #[clap(long, number_of_values = 1)]
    /// Only the specified bench target
    pub(crate) bench: Vec<String>,
    #[clap(long)]
    /// All benches
    pub(crate) benches: bool,
    #[clap(long)]
    /// All targets
    pub(crate) all_targets: bool,
    #[clap(long)]
    /// Artifacts in release mode, with optimizations
    pub(crate) release: bool,
    #[clap(long)]
    /// Artifacts with the specified profile
    pub(crate) profile: Option<String>,
    #[clap(long, number_of_values = 1)]
    /// Space-separated list of features to activate
    pub(crate) features: Vec<String>,
    #[clap(long)]
    /// Activate all available features
    pub(crate) all_features: bool,
    #[clap(long)]
    /// Do not activate the `default` feature
    pub(crate) no_default_features: bool,
    #[clap(long)]
    /// TRIPLE
    pub(crate) target: Option<String>,
    #[clap(long, parse(from_os_str))]
    /// Directory for all generated artifacts
    pub(crate) target_dir: Option<OsString>,
    #[clap(long, parse(from_os_str))]
    /// Path to Cargo.toml
    pub(crate) manifest_path: Option<OsString>,
    #[clap(long)]
    /// Error format
    pub(crate) message_format: Option<String>,
    #[clap(long, short, parse(from_occurrences))]
    /// Use verbose output (-vv very verbose/build.rs output)
    pub(crate) verbose: usize,
    #[clap(long, arg_enum, default_value = "auto")]
    /// Coloring: auto, always, never
    pub(crate) color: Coloring,
    #[clap(long)]
    /// Require Cargo.lock and cache are up to date
    pub(crate) frozen: bool,
    #[clap(long)]
    /// Require Cargo.lock is up to date
    pub(crate) locked: bool,
    #[clap(long)]
    /// Run without accessing the network
    pub(crate) offline: bool,
}

impl BuildArgs {
    #[allow(clippy::too_many_lines)]
    pub fn add_args(&self, direct_args: &mut Vec<OsString>) {
        if self.quiet {
            direct_args.push(OsString::from("--quiet"));
        }
        if let Some(jobs) = self.jobs {
            direct_args.push(OsString::from("--jobs"));
            direct_args.push(OsString::from(jobs.to_string()));
        };
        if self.lib {
            direct_args.push(OsString::from("--lib"));
        };
        if !self.bin.is_empty() {
            direct_args.push(OsString::from("--bin"));
            for bin in &self.bin {
                direct_args.push(OsString::from(bin));
            }
        }
        if self.bins {
            direct_args.push(OsString::from("--bins"));
        };
        if !self.example.is_empty() {
            direct_args.push(OsString::from("--example"));
            for example in &self.example {
                direct_args.push(OsString::from(example));
            }
        }
        if self.examples {
            direct_args.push(OsString::from("--examples"));
        };

        if !self.test.is_empty() {
            direct_args.push(OsString::from("--test"));
            for test in &self.test {
                direct_args.push(OsString::from(test));
            }
        }
        if self.tests {
            direct_args.push(OsString::from("--tests"));
        };

        if !self.bench.is_empty() {
            direct_args.push(OsString::from("--bench"));
            for bench in &self.bench {
                direct_args.push(OsString::from(bench));
            }
        }
        if self.benches {
            direct_args.push(OsString::from("--benches"));
        };

        if self.all_targets {
            direct_args.push(OsString::from("--all-targets"));
        };
        if self.release {
            direct_args.push(OsString::from("--release"));
        };

        if let Some(profile) = &self.profile {
            direct_args.push(OsString::from("--profile"));
            direct_args.push(OsString::from(profile.to_string()));
        };

        if !self.features.is_empty() {
            direct_args.push(OsString::from("--features"));
            for features in &self.features {
                direct_args.push(OsString::from(features));
            }
        }
        if self.all_features {
            direct_args.push(OsString::from("--all-features"));
        };
        if self.no_default_features {
            direct_args.push(OsString::from("--no-default-features"));
        };

        if let Some(target) = &self.target {
            direct_args.push(OsString::from("--target"));
            direct_args.push(OsString::from(target.to_string()));
        };
        if let Some(target_dir) = &self.target_dir {
            direct_args.push(OsString::from("--target-dir"));
            direct_args.push(OsString::from(target_dir));
        };
        if let Some(manifest_path) = &self.manifest_path {
            direct_args.push(OsString::from("--manifest-path"));
            direct_args.push(manifest_path.clone());
        };
        if let Some(message_format) = &self.message_format {
            direct_args.push(OsString::from("--message-format"));
            direct_args.push(OsString::from(message_format.to_string()));
        };
        if self.verbose > 0 {
            direct_args.push(OsString::from(format!("-{}", "v".repeat(self.verbose))));
        };
        if self.color != Coloring::Auto {
            direct_args.push(OsString::from("--color"));
            direct_args.push(OsString::from(
                self.color
                    .to_possible_value()
                    .expect("No skipped value allowed")
                    .get_name(),
            ));
        };
        if self.frozen {
            direct_args.push(OsString::from("--frozen"));
        };
        if self.locked {
            direct_args.push(OsString::from("--locked"));
        };
        if self.offline {
            direct_args.push(OsString::from("--offline"));
        };
    }
}
