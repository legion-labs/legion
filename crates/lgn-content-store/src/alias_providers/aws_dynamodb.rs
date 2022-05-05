use async_trait::async_trait;
use aws_sdk_dynamodb::types::Blob;
use aws_sdk_dynamodb::{model::AttributeValue, Region};
use lgn_tracing::{async_span_scope, span_fn};
use std::{fmt::Display, io::Write};

use super::{AliasReader, AliasWriter, Error, Result};
use crate::Identifier;

#[derive(Debug, Clone)]
pub struct AwsDynamoDbAliasProvider {
    region: String,
    table_name: String,
    client: aws_sdk_dynamodb::Client,
}

impl AwsDynamoDbAliasProvider {
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

    fn get_alias_id_attr(key: &[u8]) -> AttributeValue {
        let mut buf = Vec::with_capacity(1 + key.len());
        // Aliases start with a 0x01 byte.
        buf.push(1);
        buf.write_all(key).unwrap();

        AttributeValue::B(Blob::new(buf))
    }

    /// Delete the content with the specified identifier.
    ///
    /// # Errors
    ///
    /// Otherwise, any other error is returned.
    #[span_fn]
    pub async fn delete_alias(&self, key: &[u8]) -> Result<()> {
        let id_attr = Self::get_alias_id_attr(key);

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
                "failed to delete item `{:02x?}` from AWS DynamoDB: {}",
                key,
                err
            )
            .into()),
        }
    }
}

impl Display for AwsDynamoDbAliasProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AWS DynamoDB (region: {}, table: {})",
            self.region, self.table_name
        )
    }
}

#[async_trait]
impl AliasReader for AwsDynamoDbAliasProvider {
    #[span_fn]
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        let id_attr = Self::get_alias_id_attr(key);

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
                            Ok(Identifier::read_from(std::io::Cursor::new(data.into_inner()))?)
                        }
                        _ => Err(anyhow::anyhow!(
                            "failed to read item `{:02x?}` data with unexpected type from AWS DynamoDB",
                            key
                        )
                        .into()),
                    },
                    None => Err(anyhow::anyhow!(
                        "failed to read item `{:02x?}` data from AWS DynamoDB",
                        key
                    )
                    .into()),
                },
                None => Err(Error::AliasNotFound(key.into())),
            },
            Err(err) => Err(anyhow::anyhow!(
                "unexpected error while reading item `{:02x?}` from AWS DynamoDB: {}",
                key,
                err
            )
            .into()),
        }
    }
}

#[async_trait]
impl AliasWriter for AwsDynamoDbAliasProvider {
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier> {
        async_span_scope!("AwsDynamoDbAliasProvider::register_alias");

        let id_attr = Self::get_alias_id_attr(key);
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
            Ok(_) => Ok(Identifier::new_alias(key.into())),
            Err(err) => Err(anyhow::anyhow!(
                "unexpected error while writing item `{:02x?}` as {} to AWS DynamoDB: {}",
                key,
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
    async fn test_aws_dynamodb_alias_provider() {
        let table_name = "legionlabs-content-store-test";
        let alias_provider =
            AwsDynamoDbAliasProvider::new(Some("ca-central-1".to_string()), table_name)
                .await
                .unwrap();

        let uid = uuid::Uuid::new_v4();
        let key = uid.as_bytes();

        crate::alias_providers::test_alias_provider(&alias_provider, key).await;

        alias_provider
            .delete_alias(key)
            .await
            .expect("failed to delete alias");
    }
}
