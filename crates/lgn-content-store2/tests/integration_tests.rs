use std::{net::SocketAddr, sync::Arc};

use lgn_content_store2::{
    AwsS3Provider, ContentReader, ContentWriter, Error, GrpcProvider, GrpcService, Identifier,
    LocalProvider, SmallContentProvider,
};

mod common;

#[allow(clippy::wildcard_imports)]
use common::*;
use lgn_online::grpc::GrpcClient;

#[tokio::test]
async fn test_local_provider() {
    let root = tempfile::tempdir().expect("failed to create temp directory");
    let provider = LocalProvider::new(root.path())
        .await
        .expect("failed to create local provider");

    let id = Identifier::new_hash_ref_from_data(b"A");
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, "A");
    assert_read_content!(provider, id, "A");

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);
}

#[tokio::test]
async fn test_small_content_provider() {
    let root = tempfile::tempdir().expect("failed to create temp directory");
    let provider = LocalProvider::new(root.path())
        .await
        .expect("failed to create local provider");

    // Files of 1 bytes or less are stored in the identifier.
    let provider = SmallContentProvider::new_with_size_threshold(provider, 1);

    let id = assert_write_content!(provider, "A");
    assert!(id.is_data());
    assert_read_content!(provider, id, "A");

    // Another write should yield no error.
    let new_id = assert_write_content!(provider, "A");
    assert_eq!(id, new_id);

    // Since we have a hash-ref identifier, it should still not be found as the
    // SmallContentProvider would have elided the actual write before.
    let id = Identifier::new_hash_ref_from_data(b"A");
    assert_content_not_found!(provider, id);

    // Now let's try again with a larger file.
    let id = Identifier::new_hash_ref_from_data(b"AA");
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, "AA");
    assert!(id.is_hash_ref());
    assert_read_content!(provider, id, "AA");
}

#[cfg(feature = "aws")]
#[ignore]
#[tokio::test]
async fn test_aws_s3_provider() {
    let aws_s3_url = format!(
        "s3://legionlabs-ci-tests/lgn-content-store/test_aws_s3_provider/{}",
        uuid::Uuid::new_v4()
    )
    .parse()
    .unwrap();

    let provider = AwsS3Provider::new(aws_s3_url).await;
    let id = Identifier::new_hash_ref_from_data(b"A");

    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, "A");
    assert_read_content!(provider, id, "A");

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    provider
        .delete_content(&id)
        .await
        .expect("failed to delete content");
}

#[tokio::test]
async fn test_grpc_provider() {
    let root = tempfile::tempdir().expect("failed to create temp directory");
    let local_provider = LocalProvider::new(root.path())
        .await
        .expect("failed to create local provider");

    let http_server = httpmock::prelude::MockServer::start_async().await;

    let address_provider = Arc::new(FakeContentAddressProvider::new(http_server.url("/")));
    let service = GrpcService::new(local_provider, Arc::clone(&address_provider), 1);
    let service = lgn_content_store_proto::content_store_server::ContentStoreServer::new(service);
    let server = tonic::transport::Server::builder().add_service(service);

    let incoming = TcpIncoming::new().unwrap();
    let addr = incoming.addr();

    async fn f(
        socket_addr: &SocketAddr,
        http_server: &httpmock::MockServer,
        address_provider: Arc<FakeContentAddressProvider>,
    ) {
        let client = GrpcClient::new(format!("http://{}", socket_addr).parse().unwrap());
        let provider = GrpcProvider::new(client).await;

        // First we try with a small file.

        let id = Identifier::new_hash_ref_from_data(b"A");
        assert_content_not_found!(provider, id);

        let id = assert_write_content!(provider, "A");
        assert_read_content!(provider, id, "A");

        // Another write should yield no error.
        let new_id = assert_write_content!(provider, "A");
        assert_eq!(id, new_id);

        // Now let's try again with a larger file.

        let id = Identifier::new_hash_ref_from_data(b"AA");
        assert_content_not_found!(provider, id);

        let write_mock = http_server
            .mock_async(|when, then| {
                when.method("POST").path(format!("/{}/write", id));
                then.status(201).body(b"");
            })
            .await;
        let read_mock = http_server
            .mock_async(|when, then| {
                when.method("GET").path(format!("/{}/read", id));
                then.status(200).body(b"AA");
            })
            .await;

        let id = assert_write_content!(provider, "AA");
        assert_read_content!(provider, id, "AA");

        write_mock.assert();
        read_mock.assert();

        // Make sure the next write yields `Error::AlreadyExists`.
        address_provider.set_already_exists(true).await;

        // Another write should be useless.
        assert_write_avoided!(provider, &id);
    }

    loop {
        tokio::select! {
            res = async {
                server.serve_with_incoming(incoming).await
            } => panic!("server is no longer bound: {}", res.unwrap_err()),
            _ = f(&addr, &http_server, address_provider) => break
        };
    }
}
