use std::{
    env::var_os,
    process::{Command, Stdio},
};

use camino::Utf8Path;
use lgn_tracing::{error, info};

use crate::{config::Sccache, installer::Installer};

/// If the project is configured for sccache, and the env variable `SKIP_SCCACHE` is unset then returns true.
/// If the `warn_if_not_correct_location` parameter is set to true, warnings will be logged if the project is configured for sccache
/// but the `CARGO_HOME` or project root are not in the right locations.
pub fn sccache_should_run(
    workspace_root: &Utf8Path,
    sccache_config: &Option<Sccache>,
    warn_if_not_correct_location: bool,
) -> bool {
    if var_os("SKIP_SCCACHE").is_none() {
        if let Some(sccache_config) = sccache_config {
            // Are we work on items in the right location:
            // See: https://github.com/mozilla/sccache#known-caveats
            let correct_location = var_os("CARGO_HOME").unwrap_or_default()
                == sccache_config.required_cargo_home.as_str()
                && sccache_config.required_git_home.as_str() == workspace_root.as_str();
            if !correct_location && warn_if_not_correct_location {
                error!("You will not benefit from sccache in this build!!!");
                error!(
                    "To get the best experience, please move your diem source code to {} and your set your CARGO_HOME to be {}, simply export it in your .profile or .bash_rc",
                    &sccache_config.required_git_home.as_str(), &sccache_config.required_cargo_home.as_str()
                );
                error!(
                    "Current diem root is '{}',  and current CARGO_HOME is '{}'",
                    workspace_root,
                    var_os("CARGO_HOME").unwrap_or_default().to_string_lossy()
                );
            }
            correct_location
        } else {
            false
        }
    } else {
        false
    }
}

/// Logs the output of "sccache --show-stats"
pub fn log_sccache_stats() {
    println!("Sccache statistics:");
    let mut sccache = Command::new("sccache");
    sccache.arg("--show-stats");
    sccache.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let output = sccache.output();
    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                info!("sccache: {}", line);
            }
        } else {
            error!("sccache error: {}", String::from_utf8_lossy(&output.stderr));
        }
    } else {
        error!("Could not log sccache statistics: {}", output.unwrap_err());
    }
}

pub fn stop_sccache_server() {
    let mut sccache = Command::new("sccache");
    sccache.arg("--stop-server");
    sccache.stdout(Stdio::piped()).stderr(Stdio::piped());
    match sccache.output() {
        Ok(output) => {
            if output.status.success() {
                println!("Stopped already running sccache.");
            } else {
                let std_err = String::from_utf8_lossy(&output.stderr);
                //sccache will fail
                if !std_err.contains("couldn't connect to server") {
                    error!("Failed to stopped already running sccache.");
                    error!("status: {}", output.status);
                    error!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                    error!("stderr: {}", std_err);
                }
            }
        }
        Err(error) => {
            error!("Failed to stop running sccache: {}", error);
        }
    }
}

/// Starts the sccache server.
///
/// # Errors
/// If we fail to install sccache
///
pub fn apply_sccache_if_possible<'a>(
    workspace_root: &'a Utf8Path,
    installer: &'a Installer,
    sccache_config: &'a Option<Sccache>,
) -> std::result::Result<Vec<(&'a str, Option<String>)>, &'a str> {
    let mut envs = vec![];
    if sccache_should_run(workspace_root, sccache_config, var_os("CI").is_some()) {
        if let Some(sccache_config) = sccache_config {
            if !installer.install_via_cargo_if_needed("sccache") {
                return Err("Failed to install sccache, bailing");
            }
            stop_sccache_server();
            envs.push(("RUSTC_WRAPPER", Some("sccache".to_owned())));
            envs.push(("CARGO_INCREMENTAL", Some("false".to_owned())));
            envs.push(("SCCACHE_BUCKET", Some(sccache_config.bucket.clone())));
            if let Some(ssl) = &sccache_config.ssl {
                envs.push((
                    "SCCACHE_S3_USE_SSL",
                    if *ssl {
                        Some("true".to_owned())
                    } else {
                        Some("false".to_owned())
                    },
                ));
            }

            if let Some(url) = &sccache_config.endpoint {
                envs.push(("SCCACHE_ENDPOINT", Some(url.clone())));
            }

            if let Some(extra_envs) = &sccache_config.envs {
                for (key, value) in extra_envs {
                    envs.push((key, Some(value.clone())));
                }
            }

            if let Some(region) = &sccache_config.region {
                envs.push(("SCCACHE_REGION", Some(region.clone())));
            }

            if let Some(prefix) = &sccache_config.prefix {
                envs.push(("SCCACHE_S3_KEY_PREFIX", Some(prefix.clone())));
            }
            let access_key_id =
                var_os("SCCACHE_AWS_ACCESS_KEY_ID").map(|val| val.to_string_lossy().to_string());
            let access_key_secret = var_os("SCCACHE_AWS_SECRET_ACCESS_KEY")
                .map(|val| val.to_string_lossy().to_string());
            // if either the access or secret key is not set, attempt to perform a public read.
            // do not set this flag if attempting to write, as it will prevent the use of the aws creds.
            if (access_key_id.is_none() || access_key_secret.is_none())
                && sccache_config.public.unwrap_or(true)
            {
                envs.push(("SCCACHE_S3_PUBLIC", Some("true".to_owned())));
            }

            //Note: that this is also used to _unset_ AWS_ACCESS_KEY_ID & AWS_SECRET_ACCESS_KEY
            envs.push(("AWS_ACCESS_KEY_ID", access_key_id));
            envs.push(("AWS_SECRET_ACCESS_KEY", access_key_secret));
        }
    }
    Ok(envs)
}
