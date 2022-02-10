use serde::{Deserialize, Serialize};

use super::super::{
    aws_lambda::AwsLambdaDistTarget, dist_package::DistPackage, dist_target::DistTarget,
    metadata::CopyCommand,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct AwsLambdaMetadata {
    pub name: Option<String>,
    pub s3_bucket: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub s3_bucket_prefix: Option<String>,
    #[serde(default = "default_target_runtime")]
    pub target_runtime: String,
    #[serde(default)]
    pub extra_files: Vec<CopyCommand>,
    pub binary: String,
}

fn default_target_runtime() -> String {
    "x86_64-unknown-linux-musl".to_string()
}

impl AwsLambdaMetadata {
    pub(crate) fn into_dist_target<'g>(self, package: &'g DistPackage<'g>) -> DistTarget<'g> {
        DistTarget::AwsLambda(AwsLambdaDistTarget {
            package,
            metadata: self,
        })
    }
}
