use async_trait::async_trait;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::Blob;
use futures::Future;
use pin_project::pin_project;
use std::io::Cursor;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{ContentReader, ContentWriter, Error, Identifier, Result};

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
    async fn get_content_reader(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncRead + Send>>> {
        Ok(Box::pin(Cursor::new(self.get_content(id).await?)))
    }
}

#[async_trait]
impl ContentWriter for AwsDynamoDbProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncWrite + Send>>> {
        match self.get_content(id).await {
            Ok(_) => Err(Error::AlreadyExists),
            Err(Error::NotFound) => Ok(Box::pin(DynamoDbUploader::new(
                self.client.clone(),
                self.table_name.clone(),
                id.clone(),
            ))),
            Err(err) => Err(err),
        }
    }
}

#[pin_project]
struct DynamoDbUploader {
    #[pin]
    state: DynamoDbUploaderState,
}

#[allow(clippy::type_complexity)]
enum DynamoDbUploaderState {
    Writing(
        Option<(
            std::io::Cursor<Vec<u8>>,
            Identifier,
            aws_sdk_dynamodb::Client,
            String,
        )>,
    ),
    Uploading(Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'static>>),
}

impl DynamoDbUploader {
    pub fn new(client: aws_sdk_dynamodb::Client, table_name: String, id: Identifier) -> Self {
        let state = DynamoDbUploaderState::Writing(Some((
            std::io::Cursor::new(Vec::new()),
            id,
            client,
            table_name,
        )));

        Self { state }
    }

    async fn upload(
        data: Vec<u8>,
        id: Identifier,
        client: aws_sdk_dynamodb::Client,
        table_name: String,
    ) -> Result<(), std::io::Error> {
        id.matches(&data).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                anyhow::anyhow!("the data does not match the specified id: {}", err),
            )
        })?;

        let id_attr = AttributeValue::B(Blob::new(id.as_vec()));
        let data_attr = AttributeValue::B(Blob::new(data));

        match client
            .put_item()
            .table_name(table_name)
            .item("id", id_attr)
            .item("data", data_attr)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                anyhow::anyhow!(
                    "unexpected error while writing item `{}` to AWS DynamoDB: {}",
                    id,
                    err
                ),
            )),
        }
    }
}

impl AsyncWrite for DynamoDbUploader {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        let this = self.project();

        if let DynamoDbUploaderState::Writing(Some((cursor, _, _, _))) = this.state.get_mut() {
            Pin::new(cursor).poll_write(cx, buf)
        } else {
            panic!("HttpUploader::poll_write called after completion")
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();

        if let DynamoDbUploaderState::Writing(Some((cursor, _, _, _))) = this.state.get_mut() {
            Pin::new(cursor).poll_flush(cx)
        } else {
            panic!("HttpUploader::poll_flush called after completion")
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();
        let state = this.state.get_mut();

        loop {
            *state = match state {
                DynamoDbUploaderState::Writing(args) => {
                    let res = Pin::new(&mut args.as_mut().unwrap().0).poll_shutdown(cx);

                    match res {
                        Poll::Ready(Ok(())) => {
                            let (cursor, id, client, table_name) = args.take().unwrap();

                            DynamoDbUploaderState::Uploading(Box::pin(Self::upload(
                                cursor.into_inner(),
                                id,
                                client,
                                table_name,
                            )))
                        }
                        p => return p,
                    }
                }
                DynamoDbUploaderState::Uploading(call) => return Pin::new(call).poll(cx),
            };
        }
    }
}
