use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    sync::Arc,
};

use lgn_content_store2::{
    ChunkIdentifier, Chunker, ContentAddressReader, ContentAddressWriter, ContentReaderExt,
    ContentWriter, ContentWriterExt, Error, GrpcProvider, GrpcService, Identifier, LocalProvider,
    MemoryProvider, SmallContentProvider,
};

#[cfg(feature = "redis")]
use lgn_content_store2::RedisProvider;
#[cfg(feature = "aws")]
use lgn_content_store2::{AwsDynamoDbProvider, AwsS3Provider};

mod common;

#[allow(clippy::wildcard_imports)]
use common::*;
use lgn_online::grpc::GrpcClient;
use rand::Rng;

const BIG_DATA_A: [u8; 128] = [0x41; 128];
const BIGGER_DATA_A: [u8; 512] = [0x41; 512];
const BIG_DATA_B: [u8; 128] = [0x42; 128];
const BIG_DATA_X: [u8; 128] = [0x58; 128];
const BIGGER_DATA_X: [u8; 512] = [0x58; 512];
const SMALL_DATA_A: [u8; 16] = [0x41; 16];

#[test]
fn test_data_invariants() {
    assert!(Identifier::new(&BIG_DATA_A).is_hash_ref());
    assert!(Identifier::new(&BIGGER_DATA_A).is_hash_ref());
    assert!(Identifier::new(&BIG_DATA_B).is_hash_ref());
    assert!(Identifier::new(&BIG_DATA_X).is_hash_ref());
    assert!(Identifier::new(&BIGGER_DATA_X).is_hash_ref());
    assert!(Identifier::new(&SMALL_DATA_A).is_data());
}

#[tokio::test]
async fn test_chunker() {
    let provider = MemoryProvider::new();
    let chunk_size = 2;
    let chunker = Chunker::new(provider.clone()).with_chunk_size(chunk_size);

    // The content has a lots of repeats, so we expect to get a lot of identical chunks.
    let chunk_id = chunker.write_chunk(&BIG_DATA_A).await.unwrap();
    let data = chunker.read_chunk(&chunk_id).await.unwrap();

    assert_eq!(data, &BIG_DATA_A);
    assert_eq!(chunk_id.data_size(), BIG_DATA_A.len());

    // This should now exist in the provider.
    let id = Identifier::new(&BIG_DATA_A[..chunk_size]);
    assert_read_content!(provider, id, &BIG_DATA_A[..chunk_size]);

    // This chunk should be invalid as we point artificially to a an underlying
    // chunk.
    let chunk_id = ChunkIdentifier::new(2, id);

    match chunker.read_chunk(&chunk_id).await {
        Err(Error::InvalidChunkIndex(_)) => {}
        Err(err) => panic!("unexpected error: {}", err),
        Ok(_) => panic!("expected error"),
    };

    // This chunk should not exist.
    let chunk_id = ChunkIdentifier::new(2, Identifier::new(&BIG_DATA_X));

    match chunker.read_chunk(&chunk_id).await {
        Err(Error::NotFound) => {}
        Err(err) => panic!("unexpected error: {}", err),
        Ok(_) => panic!("expected error"),
    };

    // The content should have no repeats.
    let mut buf = vec![0; 512];
    let mut rng = rand::thread_rng();

    for x in &mut buf {
        *x = rng.gen::<u8>();
    }

    let chunk_id = chunker.write_chunk(&buf).await.unwrap();
    let data = chunker.read_chunk(&chunk_id).await.unwrap();

    assert_eq!(data, buf);
}

#[tokio::test]
async fn test_memory_provider() {
    let provider = MemoryProvider::new();

    let id = Identifier::new(&BIG_DATA_A);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, &BIG_DATA_A);
    assert_read_content!(provider, id, &BIG_DATA_A);

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id, fake_id],
        [Ok(&BIG_DATA_A), Err(Error::NotFound)]
    );
}

#[tokio::test]
async fn test_local_provider() {
    let root = tempfile::tempdir().expect("failed to create temp directory");
    let provider = LocalProvider::new(root.path())
        .await
        .expect("failed to create local provider");

    let id = Identifier::new(&BIG_DATA_A);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, &BIG_DATA_A);
    assert_read_content!(provider, id, &BIG_DATA_A);

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id, fake_id],
        [Ok(&BIG_DATA_A), Err(Error::NotFound)]
    );
}

#[tokio::test]
async fn test_small_content_provider() {
    let root = tempfile::tempdir().expect("failed to create temp directory");
    let provider = LocalProvider::new(root.path())
        .await
        .expect("failed to create local provider");

    let provider = SmallContentProvider::new(provider);

    let id = assert_write_content!(provider, &SMALL_DATA_A);
    assert!(id.is_data());
    assert_read_content!(provider, id, &SMALL_DATA_A);

    // Another write should yield no error.
    let new_id = assert_write_content!(provider, &SMALL_DATA_A);
    assert_eq!(id, new_id);

    // Now let's try again with a larger file.
    let id = Identifier::new(&BIG_DATA_A);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, &BIG_DATA_A);
    assert!(id.is_hash_ref());
    assert_read_content!(provider, id, &BIG_DATA_A);
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
    let id = Identifier::new(&BIG_DATA_A);

    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, &BIG_DATA_A);
    assert_read_content!(provider, id, &BIG_DATA_A);

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id.clone(), fake_id],
        [Ok(&BIG_DATA_A), Err(Error::NotFound)]
    );

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

    assert_eq!(&BIG_DATA_A, &*data);

    let id = Identifier::new(&BIG_DATA_B);

    // This read should fail as the value does not exist yet.
    assert!(provider.get_content_read_address(&id).await.is_err());

    let write_url = provider.get_content_write_address(&id).await.unwrap();
    reqwest::Client::new()
        .put(write_url)
        .body(std::str::from_utf8(&BIG_DATA_B).unwrap())
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    assert_read_content!(provider, id, &BIG_DATA_B);

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

    let uid = uuid::Uuid::new_v4();
    let mut data = Vec::new();
    std::io::Write::write_all(&mut data, &BIG_DATA_A).unwrap();
    std::io::Write::write_all(&mut data, uid.as_bytes()).unwrap();
    let data = &*data;

    let id = Identifier::new(data);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, data);
    assert_read_content!(provider, id, data);

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id.clone(), fake_id],
        [Ok(data), Err(Error::NotFound)]
    );

    provider
        .delete_content(&id)
        .await
        .expect("failed to delete content");
}

#[cfg(feature = "redis")]
#[ignore]
#[tokio::test]
async fn test_redis_provider() {
    let docker = testcontainers::clients::Cli::default();
    let redis =
        testcontainers::Docker::run(&docker, testcontainers::images::redis::Redis::default());

    let redis_url = format!("redis://localhost:{}", redis.get_host_port(6379).unwrap());
    let provider = RedisProvider::new(redis_url, "content-store")
        .await
        .expect("failed to create Redis provider");

    let uid = uuid::Uuid::new_v4();
    let mut data = Vec::new();
    std::io::Write::write_all(&mut data, &BIG_DATA_A).unwrap();
    std::io::Write::write_all(&mut data, uid.as_bytes()).unwrap();
    let data = &*data;

    let id = Identifier::new(data);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, data);
    assert_read_content!(provider, id, data);

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id.clone(), fake_id],
        [Ok(data), Err(Error::NotFound)]
    );

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
    let service = GrpcService::new(
        local_provider,
        Arc::clone(&address_provider),
        BIG_DATA_A.len(),
    );
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

        let id = Identifier::new(&BIG_DATA_A);
        assert_content_not_found!(provider, id);

        let id = assert_write_content!(provider, &BIG_DATA_A);
        assert_read_content!(provider, id, &BIG_DATA_A);

        // Another write should yield no error.
        let new_id = assert_write_content!(provider, &BIG_DATA_A);
        assert_eq!(id, new_id);

        // Now let's try again with a larger file.

        let id = Identifier::new(&BIGGER_DATA_A);

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
                httptest::matchers::request::body(std::str::from_utf8(&BIGGER_DATA_A).unwrap()),
            ])
            .respond_with(httptest::responders::status_code(201)),
        );

        http_server.expect(
            httptest::Expectation::matching(httptest::all_of![
                httptest::matchers::request::method("GET"),
                httptest::matchers::request::path(format!("/{}/read", id)),
            ])
            .respond_with(
                httptest::responders::status_code(200)
                    .body(std::str::from_utf8(&BIGGER_DATA_A).unwrap()),
            ),
        );

        let id = assert_write_content!(provider, &BIGGER_DATA_A);
        assert_read_content!(provider, &id, &BIGGER_DATA_A);

        // Make sure the next write yields `Error::AlreadyExists`.
        address_provider.set_already_exists(true).await;

        // Another write should be useless.
        assert_write_avoided!(provider, &id);

        let fake_id = Identifier::new(&BIG_DATA_X);
        let fake_id_2 = Identifier::new(&BIGGER_DATA_X);

        http_server.expect(
            httptest::Expectation::matching(httptest::all_of![
                httptest::matchers::request::method("GET"),
                httptest::matchers::request::path(format!("/{}/read", id)),
            ])
            .respond_with(
                httptest::responders::status_code(200)
                    .body(std::str::from_utf8(&BIGGER_DATA_A).unwrap()),
            ),
        );
        http_server.expect(
            httptest::Expectation::matching(httptest::all_of![
                httptest::matchers::request::method("GET"),
                httptest::matchers::request::path(format!("/{}/read", fake_id_2)),
            ])
            .respond_with(httptest::responders::status_code(404)),
        );

        // So we fetch 3 ids:
        // - The first one exists as a bigger file so we also expect a HTTP
        // fetch to happen.
        // - The second one does not exist and is not big enough to go through
        // HTTP, so should not trigger a HTTP fetch.
        // - The third one does not exist and is big enough to go through HTTP,
        // so should trigger a HTTP fetch that returns 404.

        assert_read_contents!(
            provider,
            [id, fake_id, fake_id_2],
            [
                Ok(&BIGGER_DATA_A),
                Err(Error::NotFound),
                Err(Error::NotFound)
            ]
        );
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
