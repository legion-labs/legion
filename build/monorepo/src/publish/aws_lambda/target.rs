use std::{collections::HashMap, fmt::Display, process::Command};

use camino::Utf8PathBuf;
use monorepo_base::skip_step;

use lgn_tracing::debug;

use super::super::PublishPackage;
use crate::{
    cargo::target_dir,
    context::Context,
    publish::{
        self,
        target::{
            build_zip_archive, clean, copy_binaries, copy_extra_files,
            DEFAULT_AWS_LAMBDA_S3_BUCKET_ENV_VAR_NAME,
        },
    },
    Error, Result,
};

use super::AwsLambdaMetadata;

pub struct AwsLambdaDistTarget<'g> {
    pub package: &'g PublishPackage<'g>,
    pub metadata: AwsLambdaMetadata,
}

impl Display for AwsLambdaDistTarget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "aws-lambda[{}]", self.package.name())
    }
}

impl<'g> AwsLambdaDistTarget<'g> {
    pub fn name(&self) -> &str {
        // we are supposed to have filled the name with a default value
        self.metadata.name.as_ref().unwrap()
    }

    pub fn build(&self, ctx: &Context, args: &crate::publish::Args) -> Result<()> {
        if cfg!(windows) {
            skip_step!(
                "Unsupported",
                "AWS Lambda build is not supported on Windows"
            );
            return Ok(());
        }

        let root = self.lambda_root(ctx, args)?;
        let archive = self.archive_path(ctx, args)?;

        clean(&root)?;

        let binaries: HashMap<_, _> = self
            .package
            .build_binaries(ctx, args)?
            .into_iter()
            .filter(|(name, _)| self.metadata.binary == *name)
            .collect();
        copy_binaries(&root, binaries.values())?;
        copy_extra_files(&self.metadata.extra_files, self.package.root(), &root)?;
        build_zip_archive(&root, &archive)?;

        Ok(())
    }

    pub fn publish(&self, ctx: &Context, args: &crate::publish::Args) -> Result<()> {
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

    fn upload_archive(&self, ctx: &Context, args: &crate::publish::Args) -> Result<()> {
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

    fn archive_path(&self, ctx: &Context, args: &crate::publish::Args) -> Result<Utf8PathBuf> {
        self.lambda_root(ctx, args)
            .map(|dir| dir.join(format!("aws-lambda-{}.zip", self.name())))
    }

    fn lambda_root(&self, ctx: &Context, args: &publish::Args) -> Result<Utf8PathBuf> {
        target_dir(ctx, &args.build_args).map(|dir| dir.join("aws-lambda").join(self.name()))
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
