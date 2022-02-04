use lgn_tracing::prelude::*;
use tonic::{Request, Status};

pub async fn validate_auth<T>(request: &Request<T>) -> Result<(), Status> {
    match request
        .metadata()
        .get("Authorization")
        .map(tonic::metadata::MetadataValue::to_str)
    {
        None => {
            error!("Auth: no token in request");
            Err(Status::unauthenticated(String::from("Access denied")))
        }
        Some(Err(_)) => {
            error!("Auth: error parsing token");
            Err(Status::unauthenticated(String::from("Access denied")))
        }
        Some(Ok(auth)) => {
            let url =
                "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/userInfo";
            let resp = reqwest::Client::new()
                .get(url)
                .header("Authorization", auth)
                .send()
                .await;
            if let Err(e) = resp {
                error!("Error validating credentials: {}", e);
                return Err(Status::unauthenticated(String::from("Access denied")));
            }
            let content = resp.unwrap().text().await;
            if let Err(e) = content {
                error!("Error reading user info response: {}", e);
                return Err(Status::unauthenticated(String::from("Access denied")));
            }
            let text_content = content.unwrap();
            let userinfo = serde_json::from_str::<serde_json::Value>(&text_content);
            if let Err(e) = userinfo {
                error!("Error parsing user info response: {} {}", e, text_content);
                return Err(Status::unauthenticated(String::from("Access denied")));
            }
            let email = &userinfo.unwrap()["email"];
            if !email.is_string() {
                error!("Email not found in user info response: {}", &text_content);
                return Err(Status::unauthenticated(String::from("Access denied")));
            }
            info!("authenticated user: {}", &text_content);
            Ok(())
        }
    }
}
