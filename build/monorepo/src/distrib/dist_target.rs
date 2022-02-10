use std::{fmt::Display, io::Write};

use camino::{Utf8Path, Utf8PathBuf};
use lgn_tracing::debug;
use monorepo_base::action_step;
use walkdir::WalkDir;

use crate::{context::Context, error::ErrorContext, Error};

use super::{
    aws_lambda::AwsLambdaDistTarget, docker::DockerDistTarget, metadata::CopyCommand,
    zip::ZipDistTarget, Result,
};

// Quite frankly, this structure is not used much and never in a context where
// its performance is critical. So we don't really care about the size of the
// enum.
#[allow(clippy::large_enum_variant)]
pub enum DistTarget<'g> {
    AwsLambda(AwsLambdaDistTarget<'g>),
    Docker(DockerDistTarget<'g>),
    Zip(ZipDistTarget<'g>),
}

impl DistTarget<'_> {
    pub fn build(&self, ctx: &Context, args: &super::Args) -> Result<()> {
        match self {
            DistTarget::AwsLambda(dist_target) => dist_target.build(ctx, args),
            DistTarget::Docker(dist_target) => dist_target.build(ctx, args),
            DistTarget::Zip(dist_target) => dist_target.build(ctx, args),
        }
    }

    pub fn publish(&self, ctx: &Context, args: &super::Args) -> Result<()> {
        match self {
            DistTarget::AwsLambda(dist_target) => dist_target.publish(ctx, args),
            DistTarget::Docker(dist_target) => dist_target.publish(ctx, args),
            DistTarget::Zip(dist_target) => dist_target.publish(ctx, args),
        }
    }
}

impl Display for DistTarget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistTarget::AwsLambda(dist_target) => dist_target.fmt(f),
            DistTarget::Docker(dist_target) => dist_target.fmt(f),
            DistTarget::Zip(dist_target) => dist_target.fmt(f),
        }
    }
}

pub fn build_zip_archive(root: &Utf8Path, archive: &Utf8Path) -> Result<()> {
    action_step!("Packaging", "Zipping archive {}", archive);

    let mut archive = zip::ZipWriter::new(
        std::fs::File::create(&archive)
            .map_err(|err| Error::new("failed to create zip archive file").with_source(err))?,
    );
    for entry in WalkDir::new(root) {
        let entry = entry
            .map_err(|err| Error::new("failed to walk lambda root directory").with_source(err))?;

        let file_path = entry
            .path()
            .strip_prefix(root)
            .map_err(|err| Error::new("failed to strip lambda root directory").with_source(err))?
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

pub fn copy_binaries<'p>(
    root: &Utf8Path,
    source_binaries: impl IntoIterator<Item = &'p Utf8PathBuf>,
) -> Result<()> {
    debug!("Will now copy all dependant binaries");

    std::fs::create_dir_all(&root)
        .map_err(Error::from_source)
        .with_full_context(
    "could not create `target_bin_dir` in Docker root",
    format!("The build process needed to create `{}` but it could not. You may want to verify permissions.", &root),
        )?;

    for source in source_binaries {
        let binary = source.file_name().unwrap().to_string();
        let target = root.join(&binary);

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

pub fn clean(root: &Utf8Path) -> Result<()> {
    debug!("Will now clean the build directory");

    std::fs::remove_dir_all(root).or_else(|err| match err.kind() {
        std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(Error::new("failed to clean the docker root directory").with_source(err)),
    })?;

    Ok(())
}

pub fn copy_extra_files(
    extra_files: &[CopyCommand],
    package_root: &Utf8Path,
    dist_root: &Utf8Path,
) -> Result<()> {
    debug!("Will now copy all extra files");
    for copy_command in extra_files {
        copy_command.copy_files(package_root, dist_root)?;
    }

    Ok(())
}

pub const DEFAULT_AWS_LAMBDA_S3_BUCKET_ENV_VAR_NAME: &str = "CARGO_MONOREPO_AWS_LAMBDA_S3_BUCKET";
