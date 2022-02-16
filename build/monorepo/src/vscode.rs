use std::{
    io::Write,
    process::{Command, Stdio},
};

use camino::Utf8Path;
use guppy::graph::{BuildTargetId, BuildTargetKind};
use lgn_tracing::span_fn;
use serde_json::{json, to_string_pretty};

use crate::{context::Context, Error, Result};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(long, short)]
    force: bool,
}

#[span_fn]
pub fn run(args: &Args, ctx: &Context) -> Result<()> {
    let workspace = ctx.package_graph()?.workspace();
    let mut bin_packages: Vec<_> = workspace
        .iter()
        .filter(|package| {
            package.build_targets().any(|bt| {
                (bt.kind() == BuildTargetKind::Binary
                    && matches!(bt.id(), BuildTargetId::Binary(_)))
                    || (matches!(bt.kind(), BuildTargetKind::LibraryOrExample(_))
                        && matches!(bt.id(), BuildTargetId::Example(_)))
            })
        })
        .collect();
    bin_packages.sort_by(|a, b| a.name().cmp(b.name()));

    let vscode_config = &ctx.config().vscode;
    let debugger_type = vscode_config.debugger_type.as_str();

    for package in vscode_config.overrides.keys() {
        if !workspace.contains_name(package) {
            return Err(Error::new(format!(
                "override {} is not in the workspace",
                package
            )));
        }
    }

    let mut tasks = vec![];
    tasks.push(json!({
        "type": "cargo",
        "command": "mclippy",
        "args": [
            "--workspace",
        ],
        "problemMatcher": [
            "$rustc"
        ],
        "group": {
            "kind": "build",
            "isDefault": true
        },
        "label": "Run Clippy",
    }));
    let mut configurations = vec![];
    let toolchain = toolchain_location().unwrap_or_else(|_| "not_found".into());
    for package in bin_packages {
        for target in package.build_targets() {
            if !matches!(target.id(), BuildTargetId::Binary(_))
                && !matches!(target.id(), BuildTargetId::Example(_))
            {
                continue;
            }
            let (name, display_name) = if let BuildTargetId::Binary(name) = target.id() {
                (name, name.to_string())
            } else if let BuildTargetId::Example(name) = target.id() {
                (name, format!("{} (example)", name))
            } else {
                unreachable!();
            };

            let label = format!("build: {}", display_name);
            tasks.push(json!({
                "type": "cargo",
                "command": "mbuild",
                "args": [
                    "--package",
                    package.name(),
                    if let BuildTargetId::Example(_name) = target.id() {
                        "--example"
                    } else {
                        "--bin"
                    },
                    name,
                ],
                "problemMatcher": [
                    "$rustc"
                ],
                "label": label,
                "presentation": {
                    "echo": true,
                    "reveal": "always",
                    "focus": false,
                    "panel": "shared",
                    "showReuseMessage": true,
                    "clear": true
                    }
            }));
            // part of the source map is still hardcoded
            let prelaunch_task = if vscode_config.disable_prelaunch {
                ""
            } else {
                label.as_str()
            };
            configurations.push(json!({
                "name": display_name,
                "type": debugger_type,
                "request": "launch",
                "program": format!("${{workspaceFolder}}/target/debug{}/{}.exe",
                    if let BuildTargetId::Example(_name) = target.id() {
                        "/examples"
                    } else {
                        ""
                    },
                    name
                ),
                "args": vscode_config.overrides.get(package.name()).map_or_else(
                    std::vec::Vec::new,
                    |dict| dict.get("args").unwrap_or(&vec![]).clone()
                ),
                "stopAtEntry": false,
                "cwd": "${workspaceFolder}",
                "environment": [],
                "console": "integratedTerminal",
                "sourceFileMap": {
                    "/rustc/db9d1b20bba1968c1ec1fc49616d4742c1725b4b": toolchain
                },
                "symbolSearchPath": "https://msdl.microsoft.com/download/symbols",
                "preLaunchTask":  prelaunch_task,
                "visualizerFile": "${workspaceFolder}/.vscode/legionlabs.natvis",
                "showDisplayString": true
            }));
        }
    }

    let tasks_file = ctx.workspace_root().join(".vscode").join("tasks.json");
    let tasks = json!({
        "version": "2.0.0",
        "tasks": &tasks,
    });
    let mut compounds = vec![];
    for (name, config) in &vscode_config.compounds {
        compounds.push(json!({
            "name": name,
            "configurations": config,
        }));
    }
    let launch_file = ctx.workspace_root().join(".vscode").join("launch.json");
    // hardcoded for now
    let launch = json!({
        "version": "0.2.0",
        "compounds": compounds,
        "configurations": configurations,
    });

    let settings_file = ctx.workspace_root().join(".vscode").join("settings.json");
    let settings = json!({
        "editor.formatOnSave": true,
        "files.eol": "\n",
        "rust-analyzer.checkOnSave.command": "vsclippy",
        "rust-analyzer.checkOnSave.extraArgs": [
          "--target-dir",
          "target/ra"
        ],
        "rust-analyzer.diagnostics.disabled": [
          "unresolved-macro-call"
        ],
        "css.lint.unknownAtRules": "ignore",
        "svelte.plugin.svelte.useNewTransformation": true,
        "[html]": {
          "editor.defaultFormatter": "esbenp.prettier-vscode"
        },
        "[javascript]": {
          "editor.defaultFormatter": "esbenp.prettier-vscode",
        },
        "[json]": {
          "editor.defaultFormatter": "vscode.json-language-features",
        },
        "[jsonc]": {
          "editor.defaultFormatter": "vscode.json-language-features",
        },
        "[typescript]": {
          "editor.defaultFormatter": "esbenp.prettier-vscode"
        },
        "search.exclude": {
          "pnpm-lock.yaml": true,
          "Cargo.lock": true,
        }
    });

    for (file, content) in &[
        (tasks_file, tasks),
        (launch_file, launch),
        (settings_file, settings),
    ] {
        let comment = "// This file is generated by monorepo tooling. Do not edit.";

        if std::fs::metadata(file).is_ok() {
            let generated = std::fs::read_to_string(file)
                .map_err(|e| Error::new("").with_source(e))?
                .starts_with(comment);

            if !generated && !args.force {
                return Err(Error::new(format!(
                    "Non generated file already exists: {}. Use --force to overwrite.",
                    file
                )));
            }
        }

        let mut file = std::fs::File::create(file)
            .map_err(|e| Error::new("failed to create file").with_source(e))?;

        let json = to_string_pretty(&content)
            .map_err(|e| Error::new("failed to print json").with_source(e))?;

        file.write_all(comment.as_bytes())
            .and_then(|_res| file.write_all(b"\n"))
            .and_then(|_res| file.write_all(json.as_bytes()))
            .map_err(|e| Error::new("failed to write json file").with_source(e))?;
    }

    Ok(())
}

fn toolchain_location() -> Result<String> {
    let mut cmd = Command::new("rustc");
    cmd.args(&["--print", "sysroot"]);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = cmd
        .output()
        .map_err(|err| Error::new("Failed to run `rustc`").with_source(err))?;
    if output.status.success() {
        let output = String::from_utf8_lossy(&output.stdout);
        let path = Utf8Path::new(output.trim_end_matches('\n'));
        let mut components = path.components();
        // removing the root
        components.next();
        components.next();
        let mut path = String::new();
        for component in components {
            let component = component.as_str();
            path.push('/');
            path.push_str(component);
        }
        path.push_str("/lib/rustlib/src/rust");
        Ok(path)
    } else {
        Err(Error::new("description"))
    }
}
