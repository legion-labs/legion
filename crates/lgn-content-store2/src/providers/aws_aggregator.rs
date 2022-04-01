use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use async_trait::async_trait;
use bytesize::ByteSize;

use crate::{
    traits::get_content_readers_impl, AwsDynamoDbProvider, AwsS3Provider, ContentAsyncRead,
    ContentAsyncWrite, ContentReader, ContentWriter, Identifier, Result,
};

/// An global that contains the default size above which S3 has a digressive cost for storing data.
pub const AWS_S3_THRESHOLD: ByteSize = ByteSize::kib(128);

/// A `AwsAggregator` is a provider that multiplexes S3 & `DynamoDB` together to handle the different requirements.
/// For one, we use `DynamoDB` to store payloads smaller than 128KiB.
/// Second, S3 can't support the aliases API, so we use `DynamoDB` instead for that.
#[derive(Debug, Clone)]
pub struct AwsAggregatorProvider {
    s3: AwsS3Provider,
    dynamo: AwsDynamoDbProvider,
}

impl AwsAggregatorProvider {
    /// Instantiate a new small content provider that wraps the specified
    /// provider using the default identifier size threshold.
    pub fn new(s3: AwsS3Provider, dynamo: AwsDynamoDbProvider) -> Self {
        Self { s3, dynamo }
    }
}

#[async_trait]
impl ContentReader for AwsAggregatorProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        if id.data_size() <= AWS_S3_THRESHOLD.as_u64() as usize {
            self.dynamo.get_content_reader(id).await
        } else {
            self.s3.get_content_reader(id).await
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        get_content_readers_impl(self, ids).await
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        // Always forward to DynamoDB since S3 can't implement aliases.
        self.dynamo.resolve_alias(key_space, key).await
    }
}

#[async_trait]
impl ContentWriter for AwsAggregatorProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        if id.data_size() <= AWS_S3_THRESHOLD.as_u64() as usize {
            self.dynamo.get_content_writer(id).await
        } else {
            self.s3.get_content_writer(id).await
        }
    }

    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        // Always forward to DynamoDB since S3 can't implement aliases.
        self.dynamo.register_alias(key_space, key, id).await
    }
}
