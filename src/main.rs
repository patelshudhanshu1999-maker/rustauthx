use axum::{Json, Router, routing::get};
use serde_json::json;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/health", get(health_check));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("Listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

// Handler
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({"Status": "OK"}))
}
