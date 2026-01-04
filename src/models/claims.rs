use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user id
    pub role: String, // user role: "user" or "admin"
    pub exp: usize,   // expiry timestamp
}
