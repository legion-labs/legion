use async_trait::async_trait;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::Blob;
use std::io::Cursor;

use crate::{
    traits::get_content_readers_impl, ContentAsyncRead, ContentAsyncWrite, ContentReader,
    ContentWriter, Error, Identifier, Result,
};

use super::{Uploader, UploaderImpl};

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

    /// Delete the content with the specified identifier.
    ///
    /// # Errors
    ///
    /// Otherwise, any other error is returned.
    pub async fn delete_content(&self, id: &Identifier) -> Result<()> {
        let id_attr = AttributeValue::B(Blob::new(id.as_vec()));

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
        let id_attr = AttributeValue::B(Blob::new(id.as_vec()));

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
impl ContentReader for AwsDynamoDbProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        Ok(Box::pin(Cursor::new(self.get_content(id).await?)))
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids [Identifier],
    ) -> Result<Vec<(&'ids Identifier, Result<ContentAsyncRead>)>> {
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
        let id_attr = AttributeValue::B(Blob::new(id.as_vec()));
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
