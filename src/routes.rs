use crate::middleware::auth::auth_guard;
use crate::middleware::role::RequireAdmin;
use crate::models::claims::Claims;
use axum::{
    Extension, Json, Router, middleware,
    routing::{get, post},
};
use mongodb::Client;
use serde_json::json;

async fn me(_claims: Claims) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": _claims.sub,
        "role": _claims.role
    }))
}

use crate::handlers::auth::{login, logout, refresh, register};
use crate::middleware::rate_limiter::RateLimiter;
use std::sync::Arc;

pub fn create_router(mongo_client: Client) -> Router {
    // Create rate limiters
    // Global rate limiter: 5 requests per minute for all endpoints
    let global_limiter = Arc::new(RateLimiter::new(5, 60));
    // Login rate limiter: 3 requests per minute specifically for login
    let login_limiter = Arc::new(RateLimiter::new(3, 60));

    let global_limiter_clone = global_limiter.clone();
    let login_limiter_clone = login_limiter.clone();

    Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
        // Login route with additional strict rate limiting (3 req/min)
        .route(
            "/login",
            post(login).layer(axum::middleware::from_fn(move |addr, req, next| {
                RateLimiter::middleware(login_limiter_clone.clone(), addr, req, next)
            })),
        )
        .route("/me", get(me))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route(
            "/dashboard",
            get(dashboard).layer(middleware::from_fn(auth_guard)),
        )
        // Admin-only routes
        .route("/admin/dashboard", get(admin_dashboard))
        .route("/admin/users", get(list_users))
        // Apply global rate limiter to all routes (5 req/min)
        .layer(axum::middleware::from_fn(move |addr, req, next| {
            RateLimiter::middleware(global_limiter_clone.clone(), addr, req, next)
        }))
        .layer(Extension(mongo_client))
}

async fn health(_claims: Claims) -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn dashboard(_claims: Claims) -> Json<serde_json::Value> {
    Json(json!({ "message": "Protected data" }))
}

// Admin-only handlers
async fn admin_dashboard(admin: RequireAdmin) -> Json<serde_json::Value> {
    Json(json!({
        "message": "Welcome to admin dashboard",
        "admin_user": admin.claims.sub,
        "role": admin.claims.role
    }))
}

async fn list_users(
    admin: RequireAdmin,
    Extension(client): Extension<Client>,
) -> Json<serde_json::Value> {
    let db = client.database("rustauthx");
    let users = db.collection::<mongodb::bson::Document>("users");

    // Get count of users
    let count = users.count_documents(None, None).await.unwrap_or(0);

    Json(json!({
        "message": "Admin access granted",
        "admin_user": admin.claims.sub,
        "total_users": count,
        "note": "Full user list functionality can be added here"
    }))
}
