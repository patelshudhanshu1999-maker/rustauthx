use jsonwebtoken::{DecodingKey, EncodingKey};

pub const JWT_SECRET: &str = "CHANGE_ME_SUPER_SECRET";
pub const JWT_EXP_HOURS: usize = 24;

pub fn encoding_key() -> EncodingKey {
    EncodingKey::from_secret(JWT_SECRET.as_bytes())
}

pub fn decoding_key() -> DecodingKey {
    DecodingKey::from_secret(JWT_SECRET.as_bytes())
}
