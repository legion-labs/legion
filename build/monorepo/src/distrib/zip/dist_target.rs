use std::fmt::Display;
use std::io::Write;

use camino::Utf8PathBuf;
use lgn_tracing::debug;
use monorepo_base::action_step;
use walkdir::WalkDir;

use crate::cargo::target_dir;
use crate::context::Context;
use crate::distrib::{self, dist_package::DistPackage};

use crate::error::ErrorContext;
use crate::{Error, Result};

use super::ZipMetadata;

pub struct ZipDistTarget<'g> {
    pub package: &'g DistPackage<'g>,
    pub metadata: ZipMetadata,
}

impl Display for ZipDistTarget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "zip[{}]", self.package.name())
    }
}

impl<'g> ZipDistTarget<'g> {
    pub fn build(&self, ctx: &Context, args: &distrib::Args) -> Result<()> {
        self.clean(ctx, args)?;

        let binaries = self.package.build_binaries(ctx, args)?;
        self.copy_binaries(ctx, args, binaries.values())?;
        self.copy_extra_files(ctx, args)?;

        Ok(())
    }

    pub fn publish(&self, ctx: &Context, args: &distrib::Args) -> Result<()> {
        self.build_zip_archive(ctx, args)
    }

    fn archive_path(&self, ctx: &Context, args: &crate::distrib::Args) -> Result<Utf8PathBuf> {
        Ok(target_dir(ctx, &args.build_args)?
            .join("zip")
            .join(format!("{}.zip", self.package.name())))
    }

    fn zip_root(&self, ctx: &Context, args: &distrib::Args) -> Result<Utf8PathBuf> {
        Ok(target_dir(ctx, &args.build_args)?
            .join("zip")
            .join(self.package.name()))
    }

    fn build_zip_archive(&self, ctx: &Context, args: &crate::distrib::Args) -> Result<()> {
        let archive_path = self.archive_path(ctx, args)?;

        action_step!("Packaging", "AWS Lambda archive");

        let mut archive = zip::ZipWriter::new(
            std::fs::File::create(&archive_path)
                .map_err(|err| Error::new("failed to create zip archive file").with_source(err))?,
        );

        let zip_root = &self.zip_root(ctx, args)?;

        for entry in WalkDir::new(zip_root) {
            let entry = entry.map_err(|err| {
                Error::new("failed to walk lambda root directory").with_source(err)
            })?;

            let file_path = entry
                .path()
                .strip_prefix(zip_root)
                .map_err(|err| {
                    Error::new("failed to strip lambda root directory").with_source(err)
                })?
                .display()
                .to_string();

            let metadata = std::fs::metadata(entry.path())
                .map_err(|err| Error::new("failed to get metadata").with_source(err))?;

            let options = zip::write::FileOptions::default();

            #[cfg(not(windows))]
            let options = {
                use std::os::unix::prelude::PermissionsExt;

                options.unix_permissions(metadata.permissions().mode())
            };

            if metadata.is_file() {
                archive.start_file(&file_path, options).map_err(|err| {
                    Error::new("failed to start writing file in the archive")
                        .with_source(err)
                        .with_output(format!("file path: {}", file_path))
                })?;

                let buf = std::fs::read(entry.path())
                    .map_err(|err| Error::new("failed to open file").with_source(err))?;

                archive.write_all(&buf).map_err(|err| {
                    Error::new("failed to write file in the archive")
                        .with_source(err)
                        .with_output(format!("file path: {}", file_path))
                })?;
            } else if metadata.is_dir() {
                archive.add_directory(&file_path, options).map_err(|err| {
                    Error::new("failed to add directory to the archive")
                        .with_source(err)
                        .with_output(format!("file path: {}", file_path))
                })?;
            }
        }

        archive
            .finish()
            .map_err(|err| Error::new("failed to write zip archive file").with_source(err))?;

        Ok(())
    }

    fn copy_binaries<'p>(
        &self,
        ctx: &Context,
        args: &distrib::Args,
        source_binaries: impl IntoIterator<Item = &'p Utf8PathBuf>,
    ) -> Result<()> {
        debug!("Will now copy all dependant binaries");

        let zip_root = self.zip_root(ctx, args)?;

        std::fs::create_dir_all(&zip_root)
            .map_err(Error::from_source)
            .with_full_context(
        "could not create `target_bin_dir` in Docker root",
        format!("The build process needed to create `{}` but it could not. You may want to verify permissions.", &zip_root),
            )?;

        for source in source_binaries {
            let binary = source.file_name().unwrap().to_string();
            let target = zip_root.join(&binary);

            debug!("Copying {} to {}", source, target);

            std::fs::copy(source, target)
                .map_err(Error::from_source)
                .with_full_context(
                    "failed to copy binary",
                    format!(
                        "The binary `{}` could not be copied to the Docker image.",
                        binary
                    ),
                )?;
        }

        Ok(())
    }

    fn clean(&self, ctx: &Context, args: &distrib::Args) -> Result<()> {
        debug!("Will now clean the build directory");

        std::fs::remove_dir_all(&self.zip_root(ctx, args)?).or_else(|err| match err.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(Error::new("failed to clean the docker root directory").with_source(err)),
        })?;

        Ok(())
    }

    fn copy_extra_files(&self, ctx: &Context, args: &distrib::Args) -> Result<()> {
        debug!("Will now copy all extra files");

        for copy_command in &self.metadata.extra_files {
            copy_command.copy_files(self.package.root(), &self.zip_root(ctx, args)?)?;
        }

        Ok(())
    }
}
