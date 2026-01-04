use crate::auth::jwt::decoding_key;
use crate::auth::jwt::verify_token;
use crate::handlers::blacklist::is_token_blacklisted;
use crate::models::claims::Claims;
use axum::Extension;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::{StatusCode, request::Parts};
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{Validation, decode};
use mongodb::Client;

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

        let token = bearer.token();
        println!("🔍 Checking token: {}", &token[..20]);

        // Get MongoDB client from extensions
        let Extension(client) = Extension::<Client>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Check if token is blacklisted
        let is_blacklisted = is_token_blacklisted(&client, token)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if is_blacklisted {
            println!("❌ Token is BLACKLISTED!");
            return Err(StatusCode::UNAUTHORIZED);
        }
        println!("✅ Token is NOT blacklisted");

        // Decode the JWT
        let token_data = decode::<Claims>(token, &decoding_key(), &Validation::default())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(token_data.claims)
    }
}

use axum::{body::Body, http::Request, middleware::Next, response::Response};

pub async fn auth_guard(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify the token - return 401 if invalid
    verify_token(auth).map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(next.run(req).await)
}
