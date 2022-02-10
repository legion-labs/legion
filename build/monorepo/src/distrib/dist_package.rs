use std::{
    collections::HashMap,
    io::{Read, Seek, Write},
};

use camino::{Utf8Path, Utf8PathBuf};
use guppy::{graph::BuildTargetId, PackageId};

use monorepo_base::{action_step, skip_step};

use super::{hash::HashSource, metadata::Metadata};
use crate::{
    build,
    cargo::{target_bin, target_dir},
    context::Context,
    Error, Result,
};

/// A package in the workspace.
#[derive(Clone)]
pub struct DistPackage<'g> {
    package: guppy::graph::PackageMetadata<'g>,
    metadata: Metadata,
    hash: String,
}

impl<'g> DistPackage<'g> {
    pub(super) fn new(
        ctx: &'g Context,
        package: guppy::graph::PackageMetadata<'g>,
        hash_cache: &mut HashMap<PackageId, String>,
    ) -> Result<Self> {
        assert!(
            package.in_workspace(),
            "cannot build a Package instance from a non-workspace package"
        );

        let metadata = Metadata::new(ctx, &package)?;
        let hash = HashSource::hash(&package, hash_cache, Some(&metadata.dists))?;

        Ok(Self {
            package,
            metadata,
            hash,
        })
    }

    pub(super) fn binary_targets(&self) -> Vec<&str> {
        self.package
            .build_targets()
            .filter_map(|build_target| {
                if let BuildTargetId::Binary(binary) = build_target.id() {
                    Some(binary)
                } else {
                    None
                }
            })
            .collect()
    }

    fn id(&self) -> &guppy::PackageId {
        self.package.id()
    }

    pub fn name(&self) -> &str {
        self.package.name()
    }

    pub fn version(&self) -> &semver::Version {
        self.package.version()
    }

    pub fn root(&self) -> &Utf8Path {
        self.package.manifest_path().parent().unwrap()
    }

    pub fn dist(&self, ctx: &Context, args: &super::Args) -> Result<()> {
        // if we are tagging just do that and exit
        if args.update_hash {
            return self.update_hash(args);
        }
        self.build_dist_targets(ctx, args)?;
        if args.no_dist {
            return Ok(());
        }
        self.publish_dist_targets(ctx, args)
    }

    pub fn build_dist_targets(&self, ctx: &Context, args: &super::Args) -> Result<()> {
        for dist_target in self.metadata.dist_targets(self) {
            action_step!("Building", "distribution {}", dist_target);
            let before = std::time::Instant::now();
            dist_target.build(ctx, args)?;
            let duration = before.elapsed();
            action_step!("Finished", "distribution in {:.2}s", duration.as_secs_f64());
        }

        Ok(())
    }

    pub fn publish_dist_targets(&self, ctx: &Context, args: &super::Args) -> Result<()> {
        let version = self.version();
        if let Some(current_hash) = self.metadata_hash(version) {
            if current_hash != &self.hash {
                skip_step!(
                    "Skipping",
                    "publication as current hash does not match the registered one for this version"
                );

                return Ok(());
            }
        }

        for dist_target in self.metadata.dist_targets(self) {
            action_step!("Publishing", "distribution {}", dist_target);
            let before = std::time::Instant::now();
            dist_target.publish(ctx, args)?;
            let duration = before.elapsed();
            action_step!("Finished", "publication in {:.2}s", duration.as_secs_f64());
        }

        Ok(())
    }

    pub fn build_binaries(
        &self,
        ctx: &Context,
        args: &super::Args,
    ) -> Result<HashMap<String, Utf8PathBuf>> {
        build::run(
            build::Args {
                package_args: args.package_args.clone(),
                build_args: args.build_args.clone(),
                ..build::Args::default()
            },
            ctx,
        )?;
        let mut binaries = HashMap::new();
        for binary in self.binary_targets() {
            let path = target_bin(ctx, &args.build_args, binary)?;
            if !path.exists() {
                return Err(
                    Error::new("failed to find binary").with_explanation(format!(
                        "The binary `{}` was not found in the target directory `{}`",
                        binary,
                        target_dir(ctx, &args.build_args)?
                    )),
                );
            }
            binaries.insert(binary.to_string(), path);
        }
        Ok(binaries)
    }

    pub fn metadata_hash(&self, version: &semver::Version) -> Option<&String> {
        if let Some(dist_hash) = &self.metadata.dist_hash {
            if *version == dist_hash.version {
                return Some(&dist_hash.hash);
            }
        }
        None
    }

    /// Tag the package with its current version and hash.
    ///
    /// If a tag already exist for the version, the call will fail.
    pub fn update_hash(&self, args: &super::Args) -> Result<()> {
        let version = self.version();

        if let Some(current_hash) = self.metadata_hash(version) {
            if current_hash == &self.hash {
                skip_step!(
                    "Skipping",
                    "tagging {} as a tag with an identical hash `{}` exists already",
                    self.id(),
                    self.hash,
                );

                return Ok(());
            }

            if args.force {
                action_step!("Re-tagging", "{} with hash `{}`", self.id(), &self.hash);
                Ok(())
            } else {
                Err(Error::new("tag already exists for version")
                    .with_explanation(format!(
                        "A tag for version `{}` already exists with a different hash `{}`. You may need to increment the package version number and try again.",
                        version,
                        current_hash,
                    ))
                )
            }
        } else {
            action_step!("Tagging", "{} with hash `{}`", self.id(), &self.hash);

            Ok(())
        }?;

        let manifest_path = &self.package.manifest_path();
        let mut manifest_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(manifest_path)
            .map_err(|err| Error::new("failed to open manifest").with_source(err))?;

        let mut manifest_data = String::default();

        #[allow(clippy::verbose_file_reads)]
        manifest_file
            .read_to_string(&mut manifest_data)
            .map_err(|err| Error::new("failed to read manifest").with_source(err))?;

        let mut document = manifest_data
            .parse::<toml_edit::Document>()
            .map_err(|err| Error::new("failed to parse manifest").with_source(err))?;

        document["package"]["metadata"]["monorepo"]["dist-hash"]["version"] =
            toml_edit::value(&version.to_string());
        document["package"]["metadata"]["monorepo"]["dist-hash"]["hash"] =
            toml_edit::value(&self.hash);

        manifest_file
            .seek(std::io::SeekFrom::Start(0))
            .map_err(|err| Error::new("failed to rewind in manifest file").with_source(err))?;

        manifest_file
            .write_all(document.to_string().as_bytes())
            .map_err(|err| Error::new("failed to write manifest").with_source(err))
    }
}
