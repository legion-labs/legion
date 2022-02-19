//! CI bootstrap executable
//! It's role is to compile lgn-monorepo through sccache
//!

use std::process::{Command, Stdio};

use camino::Utf8Path;
use monorepo_base::{
    config::{Sccache, Tools, MONOREPO_DEPTH},
    installer::Installer,
    sccache::{apply_sccache_if_possible, log_sccache_stats, stop_sccache_server},
};

fn main() -> std::io::Result<()> {
    let workspace_root = Utf8Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(MONOREPO_DEPTH)
        .unwrap();

    let tools = Tools::new(workspace_root).unwrap();
    let sccache = Sccache::new(workspace_root).unwrap();
    let installer = Installer::new(tools.cargo_installs);
    let sccache_envs =
        apply_sccache_if_possible(workspace_root, &installer, &sccache.sccache).unwrap();

    let mut cmd = Command::new("cargo");
    cmd.args(["build", "-p", "monorepo"]);
    for (key, option_value) in sccache_envs {
        if let Some(value) = option_value {
            cmd.env(key, value);
        } else {
            cmd.env_remove(key);
        }
    }
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let output = cmd.output();
    log_sccache_stats();
    stop_sccache_server();
    output.map(|_output| ())
}
