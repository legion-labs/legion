use serde::{Deserialize, Serialize};

use crate::distrib::{dist_package::DistPackage, dist_target::DistTarget, metadata::CopyCommand};

use super::ZipDistTarget;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZipMetadata {
    pub s3_bucket: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub s3_bucket_prefix: String,
    #[serde(default)]
    pub extra_files: Vec<CopyCommand>,
}

impl ZipMetadata {
    pub(crate) fn into_dist_target<'g>(
        self,
        name: String,
        package: &'g DistPackage<'g>,
    ) -> DistTarget<'g> {
        DistTarget::Zip(ZipDistTarget {
            name,
            package,
            metadata: self,
        })
    }
}
