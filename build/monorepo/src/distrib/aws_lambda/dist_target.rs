use std::{fmt::Display, io::Write, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use monorepo_base::{action_step, skip_step};

use lgn_tracing::debug;
use walkdir::WalkDir;

use super::super::DistPackage;
use crate::{cargo::target_dir, context::Context, distrib, error::ErrorContext, Error, Result};

use super::AwsLambdaMetadata;

pub const DEFAULT_AWS_LAMBDA_S3_BUCKET_ENV_VAR_NAME: &str = "CARGO_MONOREPO_AWS_LAMBDA_S3_BUCKET";

pub struct AwsLambdaDistTarget<'g> {
    pub package: &'g DistPackage<'g>,
    pub metadata: AwsLambdaMetadata,
}

impl Display for AwsLambdaDistTarget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "aws-lambda[{}]", self.package.name())
    }
}

impl<'g> AwsLambdaDistTarget<'g> {
    pub fn build(&self, ctx: &Context, args: &crate::distrib::Args) -> Result<()> {
        if cfg!(windows) {
            skip_step!(
                "Unsupported",
                "AWS Lambda build is not supported on Windows"
            );
            return Ok(());
        }

        self.clean(ctx, args)?;

        let binary = self.build_binary(ctx, args)?;
        self.copy_binary(ctx, args, &binary)?;
        self.copy_extra_files(ctx, args)?;

        self.build_zip_archive(ctx, args)?;

        Ok(())
    }

    pub fn publish(&self, ctx: &Context, args: &crate::distrib::Args) -> Result<()> {
        if cfg!(windows) {
            skip_step!(
                "Unsupported",
                "AWS Lambda publish is not supported on Windows"
            );
            return Ok(());
        }

        if args.build_args.mode() == "debug" && !args.force {
            skip_step!(
                "Unsupported",
                "AWS Lambda can't be published in debug mode unless `--force` is specified"
            );
            return Ok(());
        }

        self.upload_archive(ctx, args)?;

        Ok(())
    }

    fn upload_archive(&self, ctx: &Context, args: &crate::distrib::Args) -> Result<()> {
        // this is not tested, just a placeholder
        let archive_path = self.archive_path(ctx, args)?;
        let region = self.metadata.region.clone();
        let s3_bucket = self.s3_bucket()?;
        let s3_key = format!(
            "s3://{}/{}{}/v{}.zip",
            s3_bucket,
            &self.metadata.s3_bucket_prefix.as_ref().unwrap(),
            self.package.name(),
            self.package.version()
        );
        let mut cmd = Command::new("aws");
        cmd.args(&["s3", "cp", archive_path.as_str(), &s3_key]);
        if let Some(region) = region {
            cmd.args(&["--region", region.as_str()]);
        }
        let result = cmd
            .status()
            .map_err(|err| Error::new("failed to run `aws s3 cp`").with_source(err))?;

        if result.success() {
            debug!("Uploaded {} to {}", &s3_key, &s3_bucket);
            Ok(())
        } else {
            Err(Error::new("failed to upload the AWS Lambda to s3"))
        }
    }

    fn archive_path(&self, ctx: &Context, args: &crate::distrib::Args) -> Result<Utf8PathBuf> {
        self.lambda_root(ctx, args)
            .map(|dir| dir.join(format!("aws-lambda-{}.zip", self.package.name())))
    }

    fn build_zip_archive(&self, ctx: &Context, args: &crate::distrib::Args) -> Result<()> {
        let archive_path = self.archive_path(ctx, args)?;

        action_step!("Packaging", "AWS Lambda archive");

        let mut archive = zip::ZipWriter::new(
            std::fs::File::create(&archive_path)
                .map_err(|err| Error::new("failed to create zip archive file").with_source(err))?,
        );

        let lambda_root = &self.lambda_root(ctx, args)?;

        for entry in WalkDir::new(lambda_root) {
            let entry = entry.map_err(|err| {
                Error::new("failed to walk lambda root directory").with_source(err)
            })?;

            let file_path = entry
                .path()
                .strip_prefix(lambda_root)
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

    fn build_binary(&self, ctx: &Context, args: &distrib::Args) -> Result<Utf8PathBuf> {
        self.package.build_binaries(ctx, args)?.remove(&self.metadata.binary).ok_or_else(|| {
            Error::new("failed to find the specified binary in the binaries list")
                .with_explanation(format!("The configuration requires this AWS Lambda to use the `{}` binary but no such binary is declared in the crate. Was the name perhaps mistyped?", self.metadata.binary))
        })
    }

    fn copy_binary(&self, ctx: &Context, args: &distrib::Args, source: &Utf8Path) -> Result<()> {
        debug!("Will now copy the dependant binary");

        let lambda_root = self.lambda_root(ctx, args)?;

        std::fs::create_dir_all(&lambda_root)
            .map_err(Error::from_source)
            .with_full_context(
        "could not create `lambda_root` in Docker root",
        format!("The build process needed to create `{}` but it could not. You may want to verify permissions.", lambda_root),
            )?;

        // The name of the target binary is fixed to "bootstrap" by the folks at AWS.
        let target = lambda_root.join("bootstrap");

        debug!("Copying {} to {}", source, target);

        std::fs::copy(&source, target)
            .map_err(Error::from_source)
            .with_full_context(
                "failed to copy binary",
                format!(
                    "The binary `{}` could not be copied to the Docker image. Has this target been built before attempting its packaging?",
                    source,
                ),
            )?;

        Ok(())
    }

    fn clean(&self, ctx: &Context, args: &distrib::Args) -> Result<()> {
        debug!("Will now clean the build directory");

        std::fs::remove_dir_all(&self.lambda_root(ctx, args)?).or_else(|err| match err.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(Error::new("failed to clean the lambda root directory").with_source(err)),
        })?;

        Ok(())
    }

    fn s3_bucket(&self) -> Result<String> {
        match &self.metadata.s3_bucket {
            Some(s3_bucket) => Ok(s3_bucket.clone()),
            None => {
                if let Ok(s3_bucket) = std::env::var(DEFAULT_AWS_LAMBDA_S3_BUCKET_ENV_VAR_NAME) {
                    Ok(s3_bucket)
                } else {
                    Err(
                        Error::new("failed to determine AWS S3 bucket").with_explanation(format!(
                        "The field s3_bucket is empty and the environment variable {} was not set",
                        DEFAULT_AWS_LAMBDA_S3_BUCKET_ENV_VAR_NAME
                    )),
                    )
                }
            }
        }
    }

    fn lambda_root(&self, ctx: &Context, args: &distrib::Args) -> Result<Utf8PathBuf> {
        target_dir(ctx, &args.build_args)
            .map(|dir| dir.join("aws-lambda").join(self.package.name()))
    }

    fn copy_extra_files(&self, ctx: &Context, args: &distrib::Args) -> Result<()> {
        debug!("Will now copy all extra files");

        for copy_command in &self.metadata.extra_files {
            copy_command.copy_files(self.package.root(), &self.lambda_root(ctx, args)?)?;
        }

        Ok(())
    }
}

//fn is_s3_no_such_key(
//    err: aws_sdk_s3::SdkError<aws_sdk_s3::error::GetObjectError>,
//    s3_key: &str,
//    s3_bucket: &str,
//) -> Result<()> {
//    match err {
//        aws_sdk_s3::SdkError::ServiceError { err, .. } => {
//            if !err.is_no_such_key() {
//                Err(Error::from_source(err)).with_full_context(
//                    "failed to check for AWS Lambda archive existence",
//                    format!(
//                        "Could not verify the existence of the AWS Lambda \
//                                        archive `{}` in the S3 bucket `{}`. Please check \
//                                        your credentials and permissions and make sure you \
//                                        have the appropriate permissions.",
//                        s3_key, s3_bucket
//                    ),
//                )
//            } else {
//                Ok(())
//            }
//        }
//        _ => Err(Error::from_source(err)).with_full_context(
//            "failed to check for AWS Lambda archive existence",
//            format!(
//                "Could not verify the existence of the AWS Lambda \
//                                archive `{}` in the S3 bucket `{}`. Please check \
//                                your credentials and permissions and make sure you \
//                                have the appropriate permissions.",
//                s3_key, s3_bucket
//            ),
//        ),
//    }
//}
