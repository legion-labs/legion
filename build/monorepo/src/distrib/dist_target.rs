use std::fmt::Display;

use crate::context::Context;

use super::{
    aws_lambda::AwsLambdaDistTarget, docker::DockerDistTarget, zip::ZipDistTarget, Result,
};

// Quite frankly, this structure is not used much and never in a context where
// its performance is critical. So we don't really care about the size of the
// enum.
#[allow(clippy::large_enum_variant)]
pub(crate) enum DistTarget<'g> {
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
