use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    pub access_token: String,
    pub refresh_token: String,
}
