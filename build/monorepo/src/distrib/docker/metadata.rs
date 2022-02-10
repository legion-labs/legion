use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::distrib::{dist_package::DistPackage, dist_target::DistTarget, metadata::CopyCommand};

use super::DockerDistTarget;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct DockerMetadata {
    pub name: Option<String>,
    pub registry: Option<String>,
    #[serde(default = "default_target_runtime")]
    pub target_runtime: String,
    pub template: String,
    #[serde(default)]
    pub extra_files: Vec<CopyCommand>,
    #[serde(default)]
    pub allow_aws_ecr_creation: bool,
    #[serde(default = "default_target_bin_dir")]
    pub target_bin_dir: Utf8PathBuf,
}

fn default_target_bin_dir() -> Utf8PathBuf {
    Utf8PathBuf::from("/usr/local/bin")
}

fn default_target_runtime() -> String {
    "x86_64-unknown-linux-gnu".to_string()
}

impl DockerMetadata {
    pub(crate) fn into_dist_target<'g>(self, package: &'g DistPackage<'g>) -> DistTarget<'g> {
        DistTarget::Docker(DockerDistTarget {
            package,
            metadata: self,
        })
    }
}
