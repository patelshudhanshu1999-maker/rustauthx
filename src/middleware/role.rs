use crate::models::claims::Claims;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::{StatusCode, request::Parts};

/// Extractor that requires admin role
/// Usage: `async fn admin_handler(admin: RequireAdmin) -> ...`
pub struct RequireAdmin {
    pub claims: Claims,
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // First extract Claims (this validates the JWT and checks blacklist)
        let claims = Claims::from_request_parts(parts, state).await?;

        // Check if user has admin role
        if claims.role != "admin" {
            println!(
                "❌ Access denied: User {} has role '{}', expected 'admin'",
                claims.sub, claims.role
            );
            return Err(StatusCode::FORBIDDEN);
        }

        println!("✅ Admin access granted for user: {}", claims.sub);
        Ok(RequireAdmin { claims })
    }
}
