use crate::models::claims::Claims;
use axum::{
    Extension, Json, Router,
    routing::{get, post},
};
use mongodb::Client;
use serde_json::json;

async fn me(claims: Claims) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": claims.sub
    }))
}

use crate::handlers::auth::{login, register};

pub fn create_router(mongo_client: Client) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me))
        .layer(Extension(mongo_client))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
