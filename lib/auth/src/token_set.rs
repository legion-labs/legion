use serde::{Deserialize, Serialize};

/// A set of tokens.
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub token_type: String,
    pub expires_in: u64,
}
