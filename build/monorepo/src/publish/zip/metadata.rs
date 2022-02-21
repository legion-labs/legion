use serde::{Deserialize, Serialize};

use crate::publish::{metadata::CopyCommand, package::PublishPackage, target::PublishTarget};

use super::ZipPublishTarget;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ZipMetadata {
    pub name: Option<String>,
    pub s3_bucket: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub s3_bucket_prefix: Option<String>,
    #[serde(default)]
    pub extra_files: Vec<CopyCommand>,
}

impl ZipMetadata {
    pub fn into_dist_target<'g>(self, package: &'g PublishPackage<'g>) -> PublishTarget<'g> {
        PublishTarget::Zip(ZipPublishTarget {
            package,
            metadata: self,
        })
    }
}
