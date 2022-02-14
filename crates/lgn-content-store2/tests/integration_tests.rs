use lgn_content_store2::{
    AwsS3Provider, ContentReader, ContentWriter, Error, Identifier, LocalProvider,
    SmallContentProvider,
};

mod common;

#[allow(clippy::wildcard_imports)]
use common::*;

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
    let aws_s3_url = "s3://legionlabs-tests/lgn-content-store/test_aws_s3_provider"
        .parse()
        .unwrap();

    let provider = AwsS3Provider::new(aws_s3_url).await;

    let id = Identifier::new_hash_ref_from_data(b"A");

    provider
        .delete_content(&id)
        .await
        .expect("failed to delete content");

    assert_content_not_found!(provider, id);

    let id = assert_write_content!(provider, "A");
    assert_read_content!(provider, id, "A");

    // Another write should yield no error.
    let new_id = assert_write_content!(provider, "A");
    assert_eq!(id, new_id);
}
