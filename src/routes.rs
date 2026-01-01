use axum::{Extension, Json, Router, routing::get};
use mongodb::Client;
use serde_json::json;

pub fn create_router(mongo_client: Client) -> Router {
    Router::new()
        .route("/health", get(health))
        .layer(Extension(mongo_client))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok"
    }))
}
