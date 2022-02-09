//! Metadata structures for the various targets.

use std::{collections::BTreeMap, fmt::Display};

use camino::{Utf8Path, Utf8PathBuf};
use lgn_tracing::debug;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{
    aws_lambda::AwsLambdaMetadata, docker::DockerMetadata, zip::ZipMetadata, DistPackage,
    DistTarget,
};

use crate::{context::Context, Error, ErrorContext, Result};

/// The root metadata structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct Metadata {
    #[serde(flatten)]
    pub dist_targets: BTreeMap<String, DistTargetMetadata>,
    #[serde(default)]
    pub tags: BTreeMap<semver::Version, String>,
}

impl Metadata {
    pub(crate) fn new(ctx: &Context, package: &guppy::graph::PackageMetadata<'_>) -> Result<Self> {
        #[derive(Debug, Deserialize)]
        struct RootMetadata {
            #[serde(default)]
            monorepo: Metadata,
        }

        let metadata: Option<RootMetadata> =
            serde_json::from_value(package.metadata_table().clone()).map_err(|err| {
                Error::new("failed to parse metadata")
                    .with_source(err)
                    .with_explanation(format!(
                        "failed to parse the Cargo metadata for package {}",
                        package.id()
                    ))
            })?;

        let mut metadata = metadata
            .map(|metadata| metadata.monorepo)
            .unwrap_or_default();

        // Add default zip dist target if not exist
        if !metadata
            .dist_targets
            .iter()
            .any(|(_, dist_target)| matches!(dist_target, DistTargetMetadata::Zip(_)))
        {
            metadata.dist_targets.insert(
                format!("default-{}", package.name()),
                DistTargetMetadata::Zip(ZipMetadata {
                    s3_bucket: Some(ctx.config().dist.bucket.clone()),
                    region: ctx.config().dist.region.clone(),
                    s3_bucket_prefix: ctx.config().dist.prefix.clone(),
                    extra_files: vec![],
                }),
            );
        }

        Ok(metadata)
    }

    pub(crate) fn dist_targets<'g>(&self, package: &'g DistPackage<'g>) -> Vec<DistTarget<'g>> {
        self.dist_targets
            .iter()
            .map(|(name, dist_target_metadata)| {
                dist_target_metadata.to_dist_target(name.clone(), package)
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub(super) enum DistTargetMetadata {
    Docker(DockerMetadata),
    AwsLambda(AwsLambdaMetadata),
    Zip(ZipMetadata),
}

impl DistTargetMetadata {
    pub(crate) fn to_dist_target<'g>(
        &self,
        name: String,
        package: &'g DistPackage<'g>,
    ) -> DistTarget<'g> {
        match self {
            DistTargetMetadata::Docker(docker) => docker.clone().into_dist_target(name, package),
            DistTargetMetadata::AwsLambda(lambda) => lambda.clone().into_dist_target(name, package),
            DistTargetMetadata::Zip(zip) => zip.clone().into_dist_target(name, package),
        }
    }
}

impl Serialize for DistTargetMetadata {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Docker(metadata) => TargetHelper {
                target_type: TargetType::Docker,
                data: serde_json::to_value(metadata).map_err(serde::ser::Error::custom)?,
            },
            Self::AwsLambda(metadata) => TargetHelper {
                target_type: TargetType::AwsLambda,
                data: serde_json::to_value(metadata).map_err(serde::ser::Error::custom)?,
            },
            Self::Zip(metadata) => TargetHelper {
                target_type: TargetType::Zip,
                data: serde_json::to_value(metadata).map_err(serde::ser::Error::custom)?,
            },
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DistTargetMetadata {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = TargetHelper::deserialize(deserializer)?;
        match helper.target_type {
            TargetType::Docker => DockerMetadata::deserialize(helper.data)
                .map(DistTargetMetadata::Docker)
                .map_err(serde::de::Error::custom),
            TargetType::AwsLambda => AwsLambdaMetadata::deserialize(helper.data)
                .map(DistTargetMetadata::AwsLambda)
                .map_err(serde::de::Error::custom),
            TargetType::Zip => ZipMetadata::deserialize(helper.data)
                .map(DistTargetMetadata::Zip)
                .map_err(serde::de::Error::custom),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum TargetType {
    #[serde(rename = "docker")]
    Docker,
    #[serde(rename = "aws-lambda")]
    AwsLambda,
    #[serde(rename = "zip")]
    Zip,
}

#[derive(Serialize, Deserialize)]
struct TargetHelper {
    #[serde(rename = "type")]
    target_type: TargetType,
    #[serde(flatten)]
    data: serde_json::Value,
}

/// A copy command instruction.
///
/// `source` indicate the files or folders to copy, possibly using glob patterns.
/// `destination` indicates the destination of the copy operation.
///
/// If `source` is a relative path, it is relative to the current package root.
/// If `destination` is always made relative to the target root.
///
/// A copy never renames files.
#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct CopyCommand {
    pub source: Utf8PathBuf,
    pub destination: Utf8PathBuf,
}

impl CopyCommand {
    pub fn source_files(&self, package_root: &Utf8Path) -> crate::Result<Vec<Utf8PathBuf>> {
        let source = if self.source.is_relative() {
            package_root.join(&self.source).to_string()
        } else {
            self.source.to_string()
        };

        let sources = glob::glob(&source)
        .map_err(|err|
            Error::new("failed to read glob pattern")
            .with_source(err)
            .with_explanation("The specified source pattern in the copy-command could not be parsed. You may want to double-check for syntax errors.")
            .with_output(format!("Copy command: {}", self))
        )?;

        sources
            .map(|entry| entry.map(|entry| entry.to_string_lossy().as_ref().into())
                .map_err(|err|
                    Error::new("failed to resolve glob entry")
                    .with_source(err)
                    .with_explanation("The glob entry could not be resolved. This could be the result of a syntax error."))
                )
            .collect()
    }

    pub fn destination(&self, target_root: &Utf8Path) -> Utf8PathBuf {
        let destination = self
            .destination
            .strip_prefix("/")
            .unwrap_or(&self.destination);

        target_root.join(destination)
    }

    pub fn copy_files(&self, source_root: &Utf8Path, target_root: &Utf8Path) -> crate::Result<()> {
        let source_files = self.source_files(source_root)?;

        if source_files.is_empty() {
            debug!("No files to copy for `{}`. Moving on.", self);
            return Ok(());
        }

        let destination = self.destination(target_root);

        debug!(
            "Copying {} file(s) to to `{}`",
            source_files.len(),
            destination
        );

        std::fs::create_dir_all(&destination)
            .map_err(Error::from_source)
            .with_full_context(
            "could not create target directory in Docker root",
            format!("The build process needed to create `{}` but it could not. You may want to verify permissions.", &destination),
            )?;

        let options = fs_extra::dir::CopyOptions {
            overwrite: true,
            ..fs_extra::dir::CopyOptions::default()
        };

        fs_extra::copy_items(&source_files, &destination, &options)
            .map_err(|err| Error::new("failed to copy file or directory").with_source(err))?;

        Ok(())
    }
}

impl Display for CopyCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "copy '{}' -> '{}'", self.source, self.destination)
    }
}
