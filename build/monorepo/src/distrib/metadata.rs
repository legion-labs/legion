//! Metadata structures for the various targets.

use std::fmt::Display;

use camino::{Utf8Path, Utf8PathBuf};
use lgn_tracing::debug;
use serde::{Deserialize, Serialize};

use super::{
    aws_lambda::AwsLambdaMetadata, docker::DockerMetadata, zip::ZipMetadata, DistPackage,
    DistTarget,
};

use crate::{context::Context, Error, ErrorContext, Result};

/// The root metadata structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct Metadata {
    #[serde(rename = "dist")]
    pub dists: Vec<DistMetadata>,
    #[serde(default, rename = "dist-hash")]
    pub dist_hash: Option<HashMetadata>,
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

        metadata.set_defaults(ctx, package);

        Ok(metadata)
    }

    pub(super) fn dist_targets<'g>(&self, package: &'g DistPackage<'g>) -> Vec<DistTarget<'g>> {
        self.dists
            .iter()
            .map(|dist_metadata| dist_metadata.to_dist_target(package))
            .collect()
    }

    fn set_defaults(&mut self, ctx: &Context, package: &guppy::graph::PackageMetadata<'_>) {
        // Add default zip dist target if not exist
        if !self
            .dists
            .iter()
            .any(|dist_metadata| matches!(dist_metadata, DistMetadata::Zip(_)))
        {
            self.dists.push(DistMetadata::Zip(ZipMetadata {
                name: Some(package.name().to_owned()),
                s3_bucket: Some(ctx.config().dist.bucket.clone()),
                region: ctx.config().dist.region.clone(),
                s3_bucket_prefix: Some(ctx.config().dist.prefix.clone()),
                extra_files: vec![],
            }));
        }

        for dist_metadata in &mut self.dists {
            match dist_metadata {
                DistMetadata::AwsLambda(metadata) => {
                    metadata.name.get_or_insert(package.name().to_string());
                    metadata
                        .s3_bucket
                        .get_or_insert(ctx.config().dist.bucket.clone());
                    if let Some(region) = &ctx.config().dist.region {
                        metadata.region.get_or_insert(region.clone());
                    }
                    metadata
                        .s3_bucket_prefix
                        .get_or_insert(ctx.config().dist.prefix.clone());
                }
                DistMetadata::Docker(metadata) => {
                    metadata.name.get_or_insert(package.name().to_string());
                }
                DistMetadata::Zip(metadata) => {
                    metadata.name.get_or_insert(package.name().to_string());
                    metadata
                        .s3_bucket
                        .get_or_insert(ctx.config().dist.bucket.clone());
                    if let Some(region) = &ctx.config().dist.region {
                        metadata.region.get_or_insert(region.clone());
                    }
                    metadata
                        .s3_bucket_prefix
                        .get_or_insert(ctx.config().dist.prefix.clone());
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub(super) enum DistMetadata {
    #[serde(rename = "docker")]
    Docker(DockerMetadata),
    #[serde(rename = "aws-lambda")]
    AwsLambda(AwsLambdaMetadata),
    #[serde(rename = "zip")]
    Zip(ZipMetadata),
}

impl DistMetadata {
    pub fn to_dist_target<'g>(&self, package: &'g DistPackage<'g>) -> DistTarget<'g> {
        match self {
            Self::Docker(docker) => docker.clone().into_dist_target(package),
            Self::AwsLambda(lambda) => lambda.clone().into_dist_target(package),
            Self::Zip(zip) => zip.clone().into_dist_target(package),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct HashMetadata {
    pub version: semver::Version,
    pub hash: String,
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
