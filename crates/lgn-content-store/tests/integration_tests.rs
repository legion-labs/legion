use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    sync::Arc,
};

use lgn_content_store::{
    CachingProvider, ChunkIdentifier, Chunker, ContentAddressReader, ContentAddressWriter,
    ContentReaderExt, ContentTracker, ContentWriter, ContentWriterExt, DataSpace, Error,
    FallbackProvider, GrpcProvider, GrpcProviderSet, GrpcService, Identifier, LocalProvider,
    MemoryProvider, Origin, SmallContentProvider,
};

#[cfg(feature = "lru")]
use lgn_content_store::LruProvider;
#[cfg(feature = "redis")]
use lgn_content_store::RedisProvider;
#[cfg(feature = "aws")]
use lgn_content_store::{AwsDynamoDbProvider, AwsS3Provider};

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
    let chunk_size = 2;
    let chunker = Chunker::default().with_chunk_size(chunk_size);
    let provider = &MemoryProvider::new();

    // The content has a lots of repeats, so we expect to get a lot of identical chunks.
    let chunk_id = chunker.write_chunk(provider, &BIG_DATA_A).await.unwrap();
    let data = chunker.read_chunk(provider, &chunk_id).await.unwrap();

    assert_eq!(data, &BIG_DATA_A);
    assert_eq!(chunk_id.data_size(), BIG_DATA_A.len());

    // This should now exist in the provider.
    let id = Identifier::new(&BIG_DATA_A[..chunk_size]);
    assert_read_content!(provider, id, &BIG_DATA_A[..chunk_size]);

    // This chunk should be invalid as we point artificially to a an underlying
    // chunk.
    let chunk_id = ChunkIdentifier::new(2, id);

    match chunker.read_chunk(provider, &chunk_id).await {
        Err(Error::InvalidChunkIndex(_)) => {}
        Err(err) => panic!("unexpected error: {}", err),
        Ok(_) => panic!("expected error"),
    };

    // This chunk should not exist.
    let chunk_id = ChunkIdentifier::new(2, Identifier::new(&BIG_DATA_X));

    match chunker.read_chunk(provider, &chunk_id).await {
        Err(Error::IdentifierNotFound(_)) => {}
        Err(err) => panic!("unexpected error: {}", err),
        Ok(_) => panic!("expected error"),
    };

    // The content should have no repeats.
    let mut buf = vec![0; 512];
    let mut rng = rand::thread_rng();

    for x in &mut buf {
        *x = rng.gen::<u8>();
    }

    let chunk_id = chunker.write_chunk(provider, &buf).await.unwrap();
    let data = chunker.read_chunk(provider, &chunk_id).await.unwrap();

    assert_eq!(data, buf);
}

#[tokio::test]
async fn test_memory_provider() {
    let provider = MemoryProvider::new();

    assert_pop_referenced_identifiers!(provider, []);

    let id = Identifier::new(&BIG_DATA_A);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, &BIG_DATA_A);
    assert_read_content_with_origin!(provider, id, &BIG_DATA_A, Origin::Memory {});

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    assert_pop_referenced_identifiers!(provider, [&id]);

    // Popping a second time should yield no identifiers.
    assert_pop_referenced_identifiers!(provider, []);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id.clone(), fake_id],
        [
            Ok(&BIG_DATA_A),
            Err(Error::IdentifierNotFound(fake_id.clone()))
        ]
    );

    // MemoryProvider also implements AliasProvider.
    assert_alias_not_found!(provider, "space", "mykey");
    assert_write_alias!(provider, "space", "mykey", &BIG_DATA_A);
    assert_read_alias!(provider, "space", "mykey", &BIG_DATA_A);

    // Writing an alias should also have incremented the reference count.
    assert_pop_referenced_identifiers!(provider, [&id]);

    let id = assert_write_content!(provider, &BIG_DATA_A);
    assert_remove_content!(provider, &id);

    // Since we removed the content, it should not be returned.
    assert_pop_referenced_identifiers!(provider, []);
}

#[tokio::test]
async fn test_fallback_provider() {
    let main_provider = MemoryProvider::new();
    let fallback_provider = LruProvider::new(256);
    let provider = FallbackProvider::new(&main_provider, &fallback_provider);

    assert_pop_referenced_identifiers!(provider, []);

    let id = Identifier::new(&BIG_DATA_A);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, &BIG_DATA_A);
    assert_read_content_with_origin!(provider, id, &BIG_DATA_A, Origin::Memory {});

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    assert_pop_referenced_identifiers!(provider, [&id]);

    // Popping a second time should yield no identifiers.
    assert_pop_referenced_identifiers!(provider, []);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id.clone(), fake_id],
        [
            Ok(&BIG_DATA_A),
            Err(Error::IdentifierNotFound(fake_id.clone()))
        ]
    );

    assert_alias_not_found!(provider, "space", "mykey");
    assert_write_alias!(provider, "space", "mykey", &BIG_DATA_A);
    assert_read_alias!(provider, "space", "mykey", &BIG_DATA_A);

    // Writing an alias should also have incremented the reference count.
    assert_pop_referenced_identifiers!(provider, [&id]);

    let id = assert_write_content!(provider, &BIG_DATA_A);
    assert_remove_content!(provider, &id);

    // Since we removed the content, it should not be returned.
    assert_pop_referenced_identifiers!(provider, []);

    // Tests for fallback-specific behavior.

    // Let's add a value that only exists in the fallback.
    let id = assert_write_content!(fallback_provider, &BIG_DATA_B);
    assert_read_content_with_origin!(fallback_provider, id, &BIG_DATA_B, Origin::Lru {});

    // The value should be readable from the provider...
    assert_read_content_with_origin!(provider, id, &BIG_DATA_B, Origin::Lru {});
    // ... but not copied into the main provider. This is not a cache.
    assert_content_not_found!(main_provider, id);

    // Also, writing a value should not make it available in the fallback provider.
    let id = assert_write_content!(provider, &BIGGER_DATA_A);
    assert_read_content_with_origin!(provider, id, &BIGGER_DATA_A, Origin::Memory {});
    assert_content_not_found!(fallback_provider, id);

    // That is, until we pop the referenced identifiers and copy them into the fallback!
    provider
        .pop_referenced_identifiers_and_copy()
        .await
        .unwrap();

    // Now the value should be in the main provider.
    assert_read_content_with_origin!(provider, id, &BIGGER_DATA_A, Origin::Memory {});
    assert_read_content_with_origin!(fallback_provider, id, &BIGGER_DATA_A, Origin::Lru {});
}

#[cfg(feature = "lru")]
#[tokio::test]
async fn test_lru_provider() {
    let provider = LruProvider::new(2);

    let id = Identifier::new(&BIG_DATA_A);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, &BIG_DATA_A);
    assert_read_content_with_origin!(provider, id, &BIG_DATA_A, Origin::Lru {});

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id.clone(), fake_id],
        [
            Ok(&BIG_DATA_A),
            Err(Error::IdentifierNotFound(fake_id.clone()))
        ]
    );

    // Write enough content to make the LRU full.
    assert_write_content!(provider, &BIG_DATA_B);
    assert_write_content!(provider, &BIGGER_DATA_A);

    // The value should have been evicted.
    assert_content_not_found!(provider, id);

    // Rewrite the value and this time make frequent reads to avoid eviction.
    assert_write_content!(provider, &BIG_DATA_A);
    assert_read_content!(provider, id, &BIG_DATA_A);
    assert_write_content!(provider, &BIG_DATA_B);
    assert_read_content!(provider, id, &BIG_DATA_A);
    assert_write_content!(provider, &BIGGER_DATA_A);
    assert_read_content!(provider, id, &BIG_DATA_A);

    // LruProvider also implements AliasProvider.
    assert_alias_not_found!(provider, "space", "mykey");
    assert_write_alias!(provider, "space", "mykey", &BIG_DATA_A);
    assert_read_alias!(provider, "space", "mykey", &BIG_DATA_A);
}

#[tokio::test]
async fn test_caching_provider() {
    let remote_provider = Arc::new(MemoryProvider::new());
    let local_provider = Arc::new(LruProvider::new(128));
    let provider = CachingProvider::new(Arc::clone(&remote_provider), Arc::clone(&local_provider));

    let id = Identifier::new(&BIG_DATA_A);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, &BIG_DATA_A);
    assert_read_content_with_origin!(provider, id, &BIG_DATA_A, Origin::Lru {});
    assert_read_content_with_origin!(remote_provider, id, &BIG_DATA_A, Origin::Memory {});
    assert_read_content_with_origin!(local_provider, id, &BIG_DATA_A, Origin::Lru {});

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id.clone(), fake_id.clone()],
        [
            Ok(&BIG_DATA_A),
            Err(Error::IdentifierNotFound(fake_id.clone()))
        ]
    );

    // Write a value to the remote but not the cache.
    let id = assert_write_content!(remote_provider, &BIG_DATA_B);

    // The value should be copied in the cache.
    assert_read_content_with_origin!(provider, id, &BIG_DATA_B, Origin::Memory {});
    assert_read_content_with_origin!(local_provider, id, &BIG_DATA_B, Origin::Lru {});

    // Same test with a multi-read a value to the remote but not the cache.
    let id = assert_write_content!(remote_provider, &BIGGER_DATA_A);

    // The value should be copied in the cache.
    assert_read_contents!(
        provider,
        [id.clone(), fake_id],
        [
            Ok(&BIGGER_DATA_A),
            Err(Error::IdentifierNotFound(fake_id.clone()))
        ]
    );
    assert_read_content!(local_provider, id, &BIGGER_DATA_A);

    // A CachingProvider should also implement AliasProvider is the providers
    // themselves implement AliasProvider.
    assert_alias_not_found!(provider, "space", "mykey");
    assert_write_alias!(provider, "space", "mykey", &BIG_DATA_A);
    assert_read_alias!(provider, "space", "mykey", &BIG_DATA_A);
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
    assert_read_content_with_origin!(
        provider,
        id,
        &BIG_DATA_A,
        Origin::Local {
            path: root.path().join(id.to_string())
        }
    );

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id, fake_id],
        [
            Ok(&BIG_DATA_A),
            Err(Error::IdentifierNotFound(fake_id.clone()))
        ]
    );

    // LocalProvider also implements AliasProvider.
    assert_alias_not_found!(provider, "space", "mykey");
    assert_write_alias!(provider, "space", "mykey", &BIG_DATA_A);
    assert_read_alias!(provider, "space", "mykey", &BIG_DATA_A);
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
    assert_read_content_with_origin!(provider, id, &SMALL_DATA_A, Origin::InIdentifier {});

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
    let s3_prefix = format!(
        "lgn-content-store/test_aws_s3_provider/{}",
        uuid::Uuid::new_v4()
    );
    let aws_s3_url: lgn_content_store::AwsS3Url = format!("s3://legionlabs-ci-tests/{}", s3_prefix)
        .parse()
        .unwrap();

    let provider = AwsS3Provider::new(aws_s3_url.clone()).await;
    let id = Identifier::new(&BIG_DATA_A);

    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, &BIG_DATA_A);
    assert_read_content_with_origin!(
        provider,
        id,
        &BIG_DATA_A,
        Origin::AwsS3 {
            bucket_name: aws_s3_url.bucket_name.clone(),
            key: format!("{}/{}", s3_prefix, id)
        }
    );

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id.clone(), fake_id],
        [
            Ok(&BIG_DATA_A),
            Err(Error::IdentifierNotFound(fake_id.clone()))
        ]
    );

    // Make sure we can access the data through the URLs.
    let (read_url, origin) = provider
        .get_content_read_address_with_origin(&id)
        .await
        .unwrap();

    assert_eq!(
        origin,
        Origin::AwsS3 {
            bucket_name: aws_s3_url.bucket_name.clone(),
            key: format!("{}/{}", s3_prefix, id)
        }
    );

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
    assert!(provider
        .get_content_read_address_with_origin(&id)
        .await
        .is_err());

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
    let table_name = "legionlabs-content-store-test";
    let provider = AwsDynamoDbProvider::new(Some("ca-central-1".to_string()), table_name)
        .await
        .unwrap();

    let uid = uuid::Uuid::new_v4();
    let mut data = Vec::new();
    std::io::Write::write_all(&mut data, &BIG_DATA_A).unwrap();
    std::io::Write::write_all(&mut data, uid.as_bytes()).unwrap();
    let data = &*data;

    let id = Identifier::new(data);
    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, data);
    assert_read_content_with_origin!(
        provider,
        id,
        data,
        Origin::AwsDynamoDb {
            region: "ca-central-1".to_string(),
            table_name: table_name.to_string(),
            id: id.to_string()
        }
    );

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id.clone(), fake_id],
        [Ok(data), Err(Error::IdentifierNotFound(fake_id.clone()))]
    );

    // DynamoDbProvider also implements AliasProvider.
    assert_alias_not_found!(provider, "space", "mykey");
    assert_write_alias!(provider, "space", "mykey", &BIG_DATA_A);
    assert_read_alias!(provider, "space", "mykey", &BIG_DATA_A);

    provider
        .delete_content(&id)
        .await
        .expect("failed to delete content");
    provider
        .delete_alias("space", "mykey")
        .await
        .expect("failed to delete alias");
}

#[cfg(feature = "redis")]
#[ignore]
#[tokio::test]
async fn test_redis_provider() {
    let docker = testcontainers::clients::Cli::default();
    let redis =
        testcontainers::Docker::run(&docker, testcontainers::images::redis::Redis::default());

    let redis_host = format!("localhost:{}", redis.get_host_port(6379).unwrap());
    let redis_url = format!("redis://{}", redis_host);
    let key_prefix = "content-store";
    let provider = RedisProvider::new(redis_url.clone(), key_prefix)
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
    assert_read_content_with_origin!(
        provider,
        id,
        data,
        Origin::Redis {
            host: redis_host,
            key: format!("{}:content:{}", key_prefix, id)
        }
    );

    // Another write should yield no error.
    assert_write_avoided!(provider, &id);

    let fake_id = Identifier::new(&BIG_DATA_X);
    assert_read_contents!(
        provider,
        [id.clone(), fake_id],
        [Ok(data), Err(Error::IdentifierNotFound(fake_id.clone()))]
    );

    // RedisProvider also implements AliasProvider.
    assert_alias_not_found!(provider, "space", "mykey");
    assert_write_alias!(provider, "space", "mykey", &BIG_DATA_A);
    assert_read_alias!(provider, "space", "mykey", &BIG_DATA_A);

    provider
        .delete_content(&id)
        .await
        .expect("failed to delete content");
}

#[tokio::test]
async fn test_grpc_provider() {
    // To debug this test more easily, you may want to specify: RUST_LOG=httptest=debug
    let _ = pretty_env_logger::try_init();

    let provider = MemoryProvider::new();

    let http_server = httptest::Server::run();

    let address_provider = Arc::new(FakeContentAddressProvider::new(
        http_server.url("/").to_string(),
    ));
    let data_space = DataSpace::persistent();
    let providers = vec![(
        data_space.clone(),
        GrpcProviderSet {
            provider: Box::new(provider),
            address_provider: Box::new(Arc::clone(&address_provider)),
            size_threshold: BIG_DATA_A.len(),
        },
    )]
    .into_iter()
    .collect();

    let service = GrpcService::new(providers);
    let service = lgn_content_store_proto::content_store_server::ContentStoreServer::new(service);
    let server = tonic::transport::Server::builder().add_service(service);

    let incoming = TcpIncoming::new().unwrap();
    let addr = incoming.addr();

    async fn f(
        socket_addr: &SocketAddr,
        http_server: &httptest::Server,
        address_provider: Arc<FakeContentAddressProvider>,
        data_space: DataSpace,
    ) {
        let client = GrpcClient::new(format!("http://{}", socket_addr).parse().unwrap());
        let provider = GrpcProvider::new(client, data_space).await;

        // First we try with a small file.

        let id = Identifier::new(&BIG_DATA_A);
        assert_content_not_found!(provider, id);

        let id = assert_write_content!(provider, &BIG_DATA_A);
        assert_read_content_with_origin!(provider, id, &BIG_DATA_A, Origin::Memory {});

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
        assert_read_content_with_origin!(
            provider,
            &id,
            &BIGGER_DATA_A,
            Origin::Local {
                path: "fake".into()
            }
        );

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
                Err(Error::IdentifierNotFound(fake_id.clone())),
                Err(Error::IdentifierNotFound(fake_id_2.clone()))
            ]
        );

        // GrpcProvider also implements AliasProvider.
        assert_alias_not_found!(provider, "space", "mykey");
        assert_write_alias!(provider, "space", "mykey", &BIG_DATA_A);
        assert_read_alias!(provider, "space", "mykey", &BIG_DATA_A);
    }

    loop {
        tokio::select! {
            res = async {
                server.serve_with_incoming(incoming).await
            } => panic!("server is no longer bound: {}", res.unwrap_err()),
            _ = f(&addr, &http_server, address_provider, data_space) => break
        };
    }
}
