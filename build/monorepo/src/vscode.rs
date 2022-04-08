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

    let vscode_config = &ctx.config().editor.vscode;
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
    let source_map = toolchain_source_map().ok();
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
            let mut config = json!({
                "name": display_name,
                "type": debugger_type,
                "request": "launch",
                "program": format!("${{workspaceFolder}}/target/debug{}/{}{}",
                    if let BuildTargetId::Example(_name) = target.id() {
                        "/examples"
                    } else {
                        ""
                    },
                    name,
                    if cfg!(windows) {".exe"} else {""}
                ),
                "args": vscode_config.overrides.get(package.name()).map_or_else(
                    std::vec::Vec::new,
                    |dict| dict.get("args").unwrap_or(&vec![]).clone()
                ),
                "cwd": "${workspaceFolder}",
                "environment": [],
                "preLaunchTask":  prelaunch_task,
                "showDisplayString": true
            });
            if debugger_type == "lldb" {
                config["sourceLanguages"] = json!(["rust"]);
                config["stopOnEntry"] = json!(false);
                config["terminal"] = json!("integrated");
                if let Some(source_map) = source_map.clone() {
                    config["sourceMap"] = json!({ source_map.0: source_map.1 });
                }
            } else {
                config["symbolSearchPath"] = json!("https://msdl.microsoft.com/download/symbols");
                config["visualizerFile"] = json!("${workspaceFolder}/.vscode/legionlabs.natvis");
                config["stopAtEntry"] = json!(false);
                config["console"] = json!("integratedTerminal");
                if let Some(source_map) = source_map.clone() {
                    config["sourceFileMap"] = json!({ source_map.0: source_map.1 });
                }
            }
            configurations.push(config);
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
        "editor.codeActionsOnSave": {
            "source.fixAll.eslint": true
        },
        "eslint.validate": ["javascript", "typescript", "svelte"],
        "files.eol": "\n",
        "files.exclude": {
            "**/node_modules": true,
            "**/dist": true,
            "**/.pnpm-debug.log": true
        },
        "protoc": {
            "options": [
                "--proto_path=${workspaceRoot}/crates/*.proto",
            ],
        },
        "rust-analyzer.checkOnSave.command": "mclippy_ra",
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

fn toolchain_source_map() -> Result<(String, String)> {
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
        if cfg!(windows) {
            // removing the root on windows otherwise sourcemap fails
            let mut toolchain_source_path = String::new();
            components.next();
            components.next();
            for component in components {
                let component = component.as_str();
                toolchain_source_path.push('/');
                toolchain_source_path.push_str(component);
            }
            toolchain_source_path.push_str("/lib/rustlib/src/rust");
            Ok((
                "/rustc/7737e0b5c4103216d6fd8cf941b7ab9bdbaace7c".to_string(),
                toolchain_source_path,
            ))
        } else {
            // we're going to open rustc and look for a path of the form:
            // /rustc/7737e0b5c4103216d6fd8cf941b7ab9bdbaace7c/
            let rustc = path.join("bin/rustc");
            let rustc = std::fs::read(&rustc)
                .map_err(|err| Error::new("failed to read rustc").with_source(err))?;
            let rustc_subs = b"/rustc/";
            const COMMIT_SHA_SIZE: usize = 40;
            let mut cursor = 0;
            while cursor < rustc.len() - (rustc_subs.len() + COMMIT_SHA_SIZE + 1) {
                let subs_end = cursor + rustc_subs.len();
                if &rustc[cursor..subs_end] == rustc_subs
                    && rustc[subs_end + COMMIT_SHA_SIZE] == b'/'
                {
                    let build_path =
                        String::from_utf8_lossy(&rustc[cursor..subs_end + COMMIT_SHA_SIZE])
                            .to_string();
                    let mut toolchain_source_path = path.to_string();
                    toolchain_source_path.push_str("/lib/rustlib/src/rust");
                    return Ok((build_path, toolchain_source_path));
                }
                cursor += 1;
            }
            Err(Error::new("failed to find rustc"))
        }
    } else {
        Err(Error::new("failed to filnd the sysroot"))
    }
}
