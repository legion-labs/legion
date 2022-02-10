use std::fmt::Display;
use std::process::Command;

use camino::Utf8PathBuf;
use lgn_tracing::debug;

use crate::cargo::target_dir;
use crate::context::Context;
use crate::distrib::dist_target::{
    build_zip_archive, clean, copy_binaries, copy_extra_files,
    DEFAULT_AWS_LAMBDA_S3_BUCKET_ENV_VAR_NAME,
};
use crate::distrib::{self, dist_package::DistPackage};

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
    pub fn name(&self) -> &str {
        // we are supposed to have filled the name with a default value
        self.metadata.name.as_ref().unwrap()
    }

    pub fn build(&self, ctx: &Context, args: &distrib::Args) -> Result<()> {
        let root = self.zip_root(ctx, args)?;
        let archive = self.archive_path(ctx, args)?;

        clean(&root)?;

        let binaries = self.package.build_binaries(ctx, args)?;

        copy_binaries(&root, binaries.values())?;
        copy_extra_files(&self.metadata.extra_files, self.package.root(), &root)?;
        build_zip_archive(&root, &archive)?;

        Ok(())
    }

    pub fn publish(&self, ctx: &Context, args: &distrib::Args) -> Result<()> {
        let archive = self.archive_path(ctx, args)?;
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
        cmd.args(&["s3", "cp", archive.as_str(), &s3_key]);
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
        Ok(target_dir(ctx, &args.build_args)?
            .join("zip")
            .join(format!("{}.zip", self.name())))
    }

    fn zip_root(&self, ctx: &Context, args: &distrib::Args) -> Result<Utf8PathBuf> {
        Ok(target_dir(ctx, &args.build_args)?
            .join("zip")
            .join(self.name()))
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
