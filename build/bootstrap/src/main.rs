use std::process::{Command, Stdio};

use camino::Utf8Path;
use monorepo_base::{
    config::{MonorepoBaseConfig, MONOREPO_DEPTH},
    installer::Installer,
    sccache::apply_sccache_if_possible,
};

fn main() -> std::io::Result<()> {
    let workspace_root = Utf8Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(MONOREPO_DEPTH)
        .unwrap();

    let config = MonorepoBaseConfig::new(workspace_root).unwrap();
    let installer = Installer::new(config.cargo.installs.clone());
    let sccache_envs =
        apply_sccache_if_possible(workspace_root, &installer, &config.cargo.sccache).unwrap();

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
    cmd.output().map(|_output| ())
}
