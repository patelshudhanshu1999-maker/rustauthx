use crate::auth::jwt::decoding_key;
use crate::models::claims::Claims;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::{StatusCode, request::Parts};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{Validation, decode};

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| StatusCode::UNAUTHORIZED)?;

        // Decode the JWT
        let token_data = decode::<Claims>(bearer.token(), &decoding_key(), &Validation::default())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(token_data.claims)
    }
}
