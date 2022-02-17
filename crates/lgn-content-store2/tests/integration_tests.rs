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
    let new_id = assert_write_content!(provider, "A");
    assert_eq!(id, new_id);
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
    let new_id = assert_write_content!(provider, "A");
    assert_eq!(id, new_id);

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
    let address_provider = FakeContentAddressProvider::new();
    let service = GrpcService::new(local_provider, address_provider, 2);
    let service = lgn_content_store_proto::content_store_server::ContentStoreServer::new(service);
    let server = tonic::transport::Server::builder().add_service(service);

    let addr_str = get_random_localhost_addr();

    async fn f(addr_str: &str) {
        let client = GrpcClient::new(format!("http://{}", addr_str).parse().unwrap());
        let provider = GrpcProvider::new(client).await;

        let id = Identifier::new_hash_ref_from_data(b"A");
        assert_content_not_found!(provider, id);

        let id = assert_write_content!(provider, "A");
        assert_read_content!(provider, id, "A");

        // Another write should yield no error.
        let new_id = assert_write_content!(provider, "A");
        assert_eq!(id, new_id);

        // TODO: Test with a bigger content to trigger the HTTP URL mechanism.
    }

    loop {
        tokio::select! {
            res = async {
                server.serve(addr_str.parse().unwrap()).await
            } => panic!("server is no longer bound: {}", res.unwrap_err()),
            _ = f(&addr_str) => break
        };
    }
}
