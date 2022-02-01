use crate::{context::Context, Error, Result};
use std::process::Command;

// use the s3 command to copy files in or out of s3
#[allow(dead_code)]
fn s3_copy(_ctx: &Context, src: &impl AsRef<str>, dst: &impl AsRef<str>) -> Result<()> {
    let mut aws = Command::new("aws");
    aws.args(["s3", "cp", src.as_ref(), dst.as_ref()]);

    aws.status()
        .map_err(|err| Error::new("failed to run aws s3 cp").with_source(err))?;

    Ok(())
}
