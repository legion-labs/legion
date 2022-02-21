use std::{net::SocketAddr, sync::Arc};

use lgn_content_store2::{
    ContentAddressReader, ContentAddressWriter, ContentReader, ContentWriter, Error, GrpcProvider,
    GrpcService, Identifier, LocalProvider, SmallContentProvider,
};

#[cfg(feature = "redis")]
use lgn_content_store2::RedisProvider;
#[cfg(feature = "aws")]
use lgn_content_store2::{AwsDynamoDbProvider, AwsS3Provider};

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

    let id = assert_write_content!(provider, b"A");
    assert_read_content!(provider, id, b"A");

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new_hash_ref_from_data(b"XXX");
    assert_read_contents!(provider, [&id, &fake_id], [Ok(b"A"), Err(Error::NotFound)]);
}

#[tokio::test]
async fn test_small_content_provider() {
    let root = tempfile::tempdir().expect("failed to create temp directory");
    let provider = LocalProvider::new(root.path())
        .await
        .expect("failed to create local provider");

    // Files of 1 bytes or less are stored in the identifier.
    let provider = SmallContentProvider::new_with_size_threshold(provider, 1);

    let id = assert_write_content!(provider, b"A");
    assert!(id.is_data());
    assert_read_content!(provider, id, b"A");

    // Another write should yield no error.
    let new_id = assert_write_content!(provider, b"A");
    assert_eq!(id, new_id);

    // Since we have a hash-ref identifier, it should still not be found as the
    // SmallContentProvider would have elided the actual write before.
    let id = Identifier::new_hash_ref_from_data(b"A");
    assert_content_not_found!(provider, id);

    // Now let's try again with a larger file.
    let id = Identifier::new_hash_ref_from_data(b"AA");
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, b"AA");
    assert!(id.is_hash_ref());
    assert_read_content!(provider, id, b"AA");
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

    let id = assert_write_content!(provider, b"A");
    assert_read_content!(provider, id, b"A");

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new_hash_ref_from_data(b"XXX");
    assert_read_contents!(provider, [&id, &fake_id], [Ok(b"A"), Err(Error::NotFound)]);

    // Make sure we can access the data through the URLs.
    let read_url = provider.get_content_read_address(&id).await.unwrap();
    let data = reqwest::get(read_url)
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .bytes()
        .await
        .unwrap();

    assert_eq!(b"A", &*data);

    let id = Identifier::new_hash_ref_from_data(b"Hello");

    // This read should fail as the value does not exist yet.
    assert!(provider.get_content_read_address(&id).await.is_err());

    let write_url = provider.get_content_write_address(&id).await.unwrap();
    reqwest::Client::new()
        .put(write_url)
        .body("Hello")
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    assert_read_content!(provider, id, b"Hello");

    // This write should fail as the value already exists.
    assert!(provider.get_content_write_address(&id).await.is_err());

    provider
        .delete_content(&id)
        .await
        .expect("failed to delete content");
}

#[cfg(feature = "aws")]
#[ignore]
#[tokio::test]
async fn test_aws_dynamodb_provider() {
    let provider = AwsDynamoDbProvider::new("content-store-test").await;

    let data = uuid::Uuid::new_v4();
    let data = data.as_bytes();
    let id = Identifier::new_hash_ref_from_data(data);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, data);
    assert_read_content!(provider, id, data);

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new_hash_ref_from_data(b"XXX");
    assert_read_contents!(provider, [&id, &fake_id], [Ok(data), Err(Error::NotFound)]);

    provider
        .delete_content(&id)
        .await
        .expect("failed to delete content");
}

#[cfg(feature = "redis")]
#[ignore]
#[tokio::test]
async fn test_redis_provider() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let provider = RedisProvider::new(redis_url, "content-store")
        .await
        .expect("failed to create Redis provider");

    let data = uuid::Uuid::new_v4();
    let data = data.as_bytes();
    let id = Identifier::new_hash_ref_from_data(data);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, data);
    assert_read_content!(provider, id, data);

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new_hash_ref_from_data(b"XXX");
    assert_read_contents!(provider, [&id, &fake_id], [Ok(data), Err(Error::NotFound)]);

    provider
        .delete_content(&id)
        .await
        .expect("failed to delete content");
}

#[tokio::test]
async fn test_grpc_provider() {
    // To debug this test more easily, you may want to specify: RUST_LOG=httptest=debug
    let _ = pretty_env_logger::try_init();

    let root = tempfile::tempdir().expect("failed to create temp directory");
    let local_provider = LocalProvider::new(root.path())
        .await
        .expect("failed to create local provider");

    let http_server = httptest::Server::run();

    let address_provider = Arc::new(FakeContentAddressProvider::new(
        http_server.url("/").to_string(),
    ));
    let service = GrpcService::new(local_provider, Arc::clone(&address_provider), 1);
    let service = lgn_content_store_proto::content_store_server::ContentStoreServer::new(service);
    let server = tonic::transport::Server::builder().add_service(service);

    let incoming = TcpIncoming::new().unwrap();
    let addr = incoming.addr();

    async fn f(
        socket_addr: &SocketAddr,
        http_server: &httptest::Server,
        address_provider: Arc<FakeContentAddressProvider>,
    ) {
        let client = GrpcClient::new(format!("http://{}", socket_addr).parse().unwrap());
        let provider = GrpcProvider::new(client).await;

        // First we try with a small file.

        let id = Identifier::new_hash_ref_from_data(b"A");
        assert_content_not_found!(provider, id);

        let id = assert_write_content!(provider, b"A");
        assert_read_content!(provider, id, b"A");

        // Another write should yield no error.
        let new_id = assert_write_content!(provider, b"A");
        assert_eq!(id, new_id);

        // Now let's try again with a larger file.

        let id = Identifier::new_hash_ref_from_data(b"AA");

        http_server.expect(
            httptest::Expectation::matching(httptest::all_of![
                httptest::matchers::request::method("GET"),
                httptest::matchers::request::path(format!("/{}/read", id)),
            ])
            .respond_with(httptest::responders::status_code(404)),
        );

        assert_content_not_found!(provider, id);

        http_server.expect(
            httptest::Expectation::matching(httptest::all_of![
                httptest::matchers::request::method("PUT"),
                httptest::matchers::request::path(format!("/{}/write", id)),
                httptest::matchers::request::body("AA"),
            ])
            .respond_with(httptest::responders::status_code(201)),
        );

        http_server.expect(
            httptest::Expectation::matching(httptest::all_of![
                httptest::matchers::request::method("GET"),
                httptest::matchers::request::path(format!("/{}/read", id)),
            ])
            .respond_with(httptest::responders::status_code(200).body("AA")),
        );

        let id = assert_write_content!(provider, b"AA");
        assert_read_content!(provider, id, b"AA");

        // Make sure the next write yields `Error::AlreadyExists`.
        address_provider.set_already_exists(true).await;

        // Another write should be useless.
        assert_write_avoided!(provider, &id);

        let fake_id = Identifier::new_hash_ref_from_data(b"XXX");

        http_server.expect(
            httptest::Expectation::matching(httptest::all_of![
                httptest::matchers::request::method("GET"),
                httptest::matchers::request::path(format!("/{}/read", id)),
            ])
            .respond_with(httptest::responders::status_code(200).body("AA")),
        );
        http_server.expect(
            httptest::Expectation::matching(httptest::all_of![
                httptest::matchers::request::method("GET"),
                httptest::matchers::request::path(format!("/{}/read", fake_id)),
            ])
            .respond_with(httptest::responders::status_code(404)),
        );

        assert_read_contents!(provider, [&id, &fake_id], [Ok(b"AA"), Err(Error::NotFound)]);
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
