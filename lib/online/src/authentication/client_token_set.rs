use serde::{Deserialize, Serialize};

/// A set of tokens as given to clients.
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientTokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: u64,
}
