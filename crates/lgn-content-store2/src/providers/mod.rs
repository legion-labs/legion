#[cfg(feature = "aws")]
mod aws_s3;
mod local;
mod small_content;

#[cfg(feature = "aws")]
pub use aws_s3::{AwsS3Provider, AwsS3Url};
pub use local::LocalProvider;
pub use small_content::SmallContentProvider;
