use crate::middleware::auth::auth_guard;
use crate::models::claims::Claims;
use axum::{
    Extension, Json, Router, middleware,
    routing::{get, post},
};
use mongodb::Client;
use serde_json::json;

async fn me(_claims: Claims) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": _claims.sub
    }))
}

use crate::handlers::auth::{login, logout, refresh, register};

pub fn create_router(mongo_client: Client) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route(
            "/dashboard",
            get(dashboard).layer(middleware::from_fn(auth_guard)),
        )
        .layer(Extension(mongo_client))
}

async fn health(_claims: Claims) -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn dashboard(_claims: Claims) -> Json<serde_json::Value> {
    Json(json!({ "message": "Protected data" }))
}
