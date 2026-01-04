use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::claims::Claims;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

pub const JWT_SECRET: &[u8] = b"CHANGE_ME_SUPER_SECRET";

// pub fn encoding_key() -> EncodingKey {
//     EncodingKey::from_secret(JWT_SECRET)
// }

pub fn decoding_key() -> DecodingKey {
    DecodingKey::from_secret(JWT_SECRET)
}

pub fn generate_token(user_id: &str) -> String {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 900; // 15 min

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .unwrap()
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
