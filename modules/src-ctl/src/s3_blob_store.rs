use http::Uri;

pub fn validate_connection_to_bucket(s3uri: &str) -> Result<(), String> {
    let uri = s3uri.parse::<Uri>().unwrap();
    let bucket_name = uri.host().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let client = s3::Client::from_env();
    let req = client.get_bucket_location().bucket(bucket_name);
    if let Err(e) = runtime.block_on(req.send()) {
        return Err(format!("Error connecting to bucket {}: {}", s3uri, e));
    }
    Ok(())
}
