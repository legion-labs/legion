use async_trait::async_trait;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::types::Blob;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Write},
};

use crate::{
    traits::get_content_readers_impl, AliasRegisterer, AliasResolver, ContentAsyncRead,
    ContentAsyncWrite, ContentReader, ContentWriter, Error, Identifier, Result,
};

use super::{Uploader, UploaderImpl};

#[derive(Debug, Clone)]
pub struct AwsDynamoDbProvider {
    table_name: String,
    client: aws_sdk_dynamodb::Client,
}

impl AwsDynamoDbProvider {
    /// Generates a new AWS `DynamoDB` provider using the specified table.
    ///
    /// The default AWS configuration is used.
    pub async fn new(table_name: impl Into<String>) -> Self {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_dynamodb::Client::new(&config);
        let table_name = table_name.into();

        Self { table_name, client }
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
                None => Err(Error::NotFound),
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

#[async_trait]
impl AliasResolver for AwsDynamoDbProvider {
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
                None => Err(Error::NotFound),
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
impl AliasRegisterer for AwsDynamoDbProvider {
    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
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

#[async_trait]
impl ContentReader for AwsDynamoDbProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        Ok(Box::pin(Cursor::new(self.get_content(id).await?)))
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        get_content_readers_impl(self, ids).await
    }
}

#[async_trait]
impl ContentWriter for AwsDynamoDbProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        match self.get_content(id).await {
            Ok(_) => Err(Error::AlreadyExists),
            Err(Error::NotFound) => Ok(Box::pin(DynamoDbUploader::new(
                id.clone(),
                DynamoDbUploaderImpl {
                    client: self.client.clone(),
                    table_name: self.table_name.clone(),
                },
            ))),
            Err(err) => Err(err),
        }
    }
}

type DynamoDbUploader = Uploader<DynamoDbUploaderImpl>;

struct DynamoDbUploaderImpl {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
}

#[async_trait]
impl UploaderImpl for DynamoDbUploaderImpl {
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()> {
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
