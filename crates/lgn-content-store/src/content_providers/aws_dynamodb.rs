use async_trait::async_trait;
use aws_sdk_dynamodb::types::Blob;
use aws_sdk_dynamodb::{model::AttributeValue, Region};
use lgn_tracing::{async_span_scope, span_fn};
use std::{fmt::Display, io::Cursor};

use super::{
    ContentAsyncReadWithOriginAndSize, ContentAsyncWrite, ContentReader, ContentWriter, Error,
    HashRef, Origin, Result, WithOriginAndSize,
};

use super::{Uploader, UploaderImpl};

#[derive(Debug, Clone)]
pub struct AwsDynamoDbContentProvider {
    region: String,
    table_name: String,
    client: aws_sdk_dynamodb::Client,
}

impl AwsDynamoDbContentProvider {
    /// Generates a new AWS `DynamoDB` provider using the specified table.
    ///
    /// The default AWS configuration is used.
    ///
    /// # Errors
    ///
    /// If the specified or configured region is not valid, an error is
    /// returned.
    #[span_fn]
    pub async fn new(region: Option<String>, table_name: impl Into<String>) -> Result<Self> {
        let config = aws_config::from_env();

        let config = if let Some(region) = region {
            let region = Region::new(region);

            config.region(region)
        } else {
            config
        }
        .load()
        .await;

        let region = config
            .region()
            .ok_or_else(|| Error::Configuration("no AWS region was defined".to_string()))?
            .to_string();

        let client = aws_sdk_dynamodb::Client::new(&config);
        let table_name = table_name.into();

        Ok(Self {
            region,
            table_name,
            client,
        })
    }

    fn get_content_id_attr(id: &HashRef) -> AttributeValue {
        let mut buf = Vec::with_capacity(1 + id.bytes_len());
        // HashRefs start with a 0x00 byte.
        buf.push(0);
        id.write_to(&mut buf).unwrap();

        AttributeValue::B(Blob::new(buf))
    }

    /// Delete the content with the specified identifier.
    ///
    /// # Errors
    ///
    /// Otherwise, any other error is returned.
    #[span_fn]
    pub async fn delete_content(&self, id: &HashRef) -> Result<()> {
        let id_attr = Self::get_content_id_attr(id);

        match self
            .client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", id_attr)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                Err(
                    anyhow::anyhow!("failed to delete item `{}` from AWS DynamoDB: {}", id, err)
                        .into(),
                )
            }
        }
    }

    #[span_fn]
    async fn get_content(&self, id: &HashRef) -> Result<Vec<u8>> {
        let id_attr = Self::get_content_id_attr(id);

        match self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", id_attr)
            .send()
            .await
        {
            Ok(output) => match output.item {
                Some(mut attrs) => match attrs.remove("data") {
                    Some(data) => match data {
                        AttributeValue::B(data) => Ok(data.into_inner()),
                        _ => Err(anyhow::anyhow!(
                            "failed to read item `{}` data with unexpected type from AWS DynamoDB",
                            id
                        )
                        .into()),
                    },
                    None => Err(anyhow::anyhow!(
                        "failed to read item `{}` data from AWS DynamoDB",
                        id
                    )
                    .into()),
                },
                None => Err(Error::HashRefNotFound(id.clone())),
            },
            Err(err) => Err(anyhow::anyhow!(
                "unexpected error while reading item `{}` from AWS DynamoDB: {}",
                id,
                err
            )
            .into()),
        }
    }
}

impl Display for AwsDynamoDbContentProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AWS DynamoDB (table: {})", self.table_name)
    }
}

#[async_trait]
impl ContentReader for AwsDynamoDbContentProvider {
    #[span_fn]
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        let origin = Origin::AwsDynamoDb {
            region: self.region.clone(),
            table_name: self.table_name.clone(),
            id: id.to_string(),
        };

        Ok(Cursor::new(self.get_content(id).await?).with_origin_and_size(origin, id.data_size()))
    }
}

#[async_trait]
impl ContentWriter for AwsDynamoDbContentProvider {
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        async_span_scope!("AwsDynamoDbProvider::get_content_writer");

        match self.get_content(id).await {
            Ok(_) => Err(Error::HashRefAlreadyExists(id.clone())),
            Err(Error::HashRefNotFound(_)) => {
                Ok(Box::pin(DynamoDbUploader::new(DynamoDbUploaderImpl {
                    client: self.client.clone(),
                    table_name: self.table_name.clone(),
                    id: id.clone(),
                })))
            }
            Err(err) => Err(err),
        }
    }
}

type DynamoDbUploader = Uploader<DynamoDbUploaderImpl>;

#[derive(Debug)]
struct DynamoDbUploaderImpl {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
    id: HashRef,
}

#[async_trait]
impl UploaderImpl for DynamoDbUploaderImpl {
    async fn upload(self, data: Vec<u8>) -> Result<()> {
        async_span_scope!("AwsDynamoDbProvider::upload");

        let id = HashRef::new_from_data(&data);

        if id != self.id {
            return Err(Error::UnexpectedHashRef {
                expected: self.id,
                actual: id,
            });
        }

        let id_attr = AwsDynamoDbContentProvider::get_content_id_attr(&id);
        let data_attr = AttributeValue::B(Blob::new(data));

        match self
            .client
            .put_item()
            .table_name(self.table_name)
            .item("id", id_attr)
            .item("data", data_attr)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow::anyhow!(
                "unexpected error while writing item `{}` to AWS DynamoDB: {}",
                id,
                err
            )
            .into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_aws_dynamodb_content_provider() {
        let table_name = "legionlabs-content-store-test";
        let content_provider =
            AwsDynamoDbContentProvider::new(Some("ca-central-1".to_string()), table_name)
                .await
                .unwrap();

        let data = &*{
            let mut data = Vec::new();
            let uid = uuid::Uuid::new_v4();

            const BIG_DATA_A: [u8; 128] = [0x41; 128];
            std::io::Write::write_all(&mut data, &BIG_DATA_A).unwrap();
            std::io::Write::write_all(&mut data, uid.as_bytes()).unwrap();

            data
        };

        let origin = Origin::AwsDynamoDb {
            region: "ca-central-1".to_string(),
            table_name: table_name.to_string(),
            id: HashRef::new_from_data(data).to_string(),
        };

        let id =
            crate::content_providers::test_content_provider(&content_provider, data, origin).await;

        content_provider
            .delete_content(&id)
            .await
            .expect("failed to delete content");
    }
}
