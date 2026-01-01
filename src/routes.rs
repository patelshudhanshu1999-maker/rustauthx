use axum::{
    Extension, Json, Router,
    routing::{get, post},
};
use mongodb::Client;
use serde_json::json;

use crate::handlers::auth::register;

pub fn create_router(mongo_client: Client) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
        .layer(Extension(mongo_client))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
