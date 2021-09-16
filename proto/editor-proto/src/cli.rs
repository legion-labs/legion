use editor_client::EditorClient;

tonic::include_proto!("editor");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EditorClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(UpdatePropertiesRequest {
        update_id: 1,
        property_path: "/my/property/path".into(),
        value: "foo".into(),
    });

    let response = client.update_properties(request).await?;

    println!(
        "Property was updated ({}).",
        response.into_inner().update_id
    );

    Ok(())
}
