use async_trait::async_trait;
use aws_sdk_dynamodb::types::Blob;
use aws_sdk_dynamodb::{model::AttributeValue, Region};
use lgn_tracing::{async_span_scope, span_fn};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    io::{Cursor, Write},
};

use crate::{
    traits::{get_content_readers_impl, WithOrigin},
    ContentAsyncReadWithOrigin, ContentAsyncWrite, ContentReader, ContentWriter, Error, Identifier,
    Origin, Result,
};

use super::{Uploader, UploaderImpl};

#[derive(Debug, Clone)]
pub struct AwsDynamoDbProvider {
    region: String,
    table_name: String,
    client: aws_sdk_dynamodb::Client,
}

impl AwsDynamoDbProvider {
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
            .ok_or_else(|| Error::Unknown(anyhow::anyhow!("no AWS region was defined")))?
            .to_string();

        let client = aws_sdk_dynamodb::Client::new(&config);
        let table_name = table_name.into();

        Ok(Self {
            region,
            table_name,
            client,
        })
    }

    fn get_content_id_attr(id: &Identifier) -> AttributeValue {
        let mut buf = Vec::with_capacity(1 + id.bytes_len());
        // Identifiers start with a 0x00 byte.
        buf.push(0);
        id.write_to(&mut buf).unwrap();

        AttributeValue::B(Blob::new(buf))
    }

    fn get_alias_id_attr(key_space: &str, key: &str) -> AttributeValue {
        let mut buf = Vec::with_capacity(1 + key_space.len() + 1 + key.len());
        // Aliases start with a 0x01 byte.
        buf.push(1);
        write!(&mut buf, "{}:{}", key_space, key).unwrap();

        AttributeValue::B(Blob::new(buf))
    }

    /// Delete the content with the specified identifier.
    ///
    /// # Errors
    ///
    /// Otherwise, any other error is returned.
    #[span_fn]
    pub async fn delete_alias(&self, key_space: &str, key: &str) -> Result<()> {
        let id_attr = Self::get_alias_id_attr(key_space, key);

        match self
            .client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", id_attr)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow::anyhow!(
                "failed to delete item `{}/{}` from AWS DynamoDB: {}",
                key_space,
                key,
                err
            )
            .into()),
        }
    }

    /// Delete the content with the specified identifier.
    ///
    /// # Errors
    ///
    /// Otherwise, any other error is returned.
    #[span_fn]
    pub async fn delete_content(&self, id: &Identifier) -> Result<()> {
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
    async fn get_content(&self, id: &Identifier) -> Result<Vec<u8>> {
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
                None => Err(Error::IdentifierNotFound(id.clone())),
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

impl Display for AwsDynamoDbProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AWS DynamoDB (table: {})", self.table_name)
    }
}

#[async_trait]
impl ContentReader for AwsDynamoDbProvider {
    #[span_fn]
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin> {
        let origin = Origin::AwsDynamoDb {
            region: self.region.clone(),
            table_name: self.table_name.clone(),
            id: id.to_string(),
        };

        Ok(Cursor::new(self.get_content(id).await?).with_origin(origin))
    }

    #[span_fn]
    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
        get_content_readers_impl(self, ids).await
    }

    #[span_fn]
    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        let id_attr = Self::get_alias_id_attr(key_space, key);

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
                        AttributeValue::B(data) => {
                            Identifier::read_from(std::io::Cursor::new(data.into_inner()))
                        }
                        _ => Err(anyhow::anyhow!(
                            "failed to read item `{}/{}` data with unexpected type from AWS DynamoDB",
                            key_space, key
                        )
                        .into()),
                    },
                    None => Err(anyhow::anyhow!(
                        "failed to read item `{}/{}` data from AWS DynamoDB",
                        key_space, key
                    )
                    .into()),
                },
                None => Err(Error::AliasNotFound{
                    key_space: key_space.to_string(),
                    key: key.to_string(),
                }),
            },
            Err(err) => Err(anyhow::anyhow!(
                "unexpected error while reading item `{}/{}` from AWS DynamoDB: {}",
                key_space, key,
                err
            )
            .into()),
        }
    }
}

#[async_trait]
impl ContentWriter for AwsDynamoDbProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        async_span_scope!("AwsDynamoDbProvider::get_content_writer");

        match self.get_content(id).await {
            Ok(_) => Err(Error::IdentifierAlreadyExists(id.clone())),
            Err(Error::IdentifierNotFound(_)) => Ok(Box::pin(DynamoDbUploader::new(
                id.clone(),
                DynamoDbUploaderImpl {
                    client: self.client.clone(),
                    table_name: self.table_name.clone(),
                },
            ))),
            Err(err) => Err(err),
        }
    }

    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        async_span_scope!("AwsDynamoDbProvider::register_alias");

        let id_attr = Self::get_alias_id_attr(key_space, key);
        let data_attr = AttributeValue::B(Blob::new(id.as_vec()));

        match self
            .client
            .put_item()
            .table_name(&self.table_name)
            .item("id", id_attr)
            .item("data", data_attr)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow::anyhow!(
                "unexpected error while writing item `{}/{}` as {} to AWS DynamoDB: {}",
                key_space,
                key,
                id,
                err
            )
            .into()),
        }
    }
}

type DynamoDbUploader = Uploader<DynamoDbUploaderImpl>;

#[derive(Debug)]
struct DynamoDbUploaderImpl {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
}

#[async_trait]
impl UploaderImpl for DynamoDbUploaderImpl {
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()> {
        async_span_scope!("AwsDynamoDbProvider::upload");

        let id_attr = AwsDynamoDbProvider::get_content_id_attr(&id);
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
