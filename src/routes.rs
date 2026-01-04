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
        // Admin-only routes
        .route("/admin/dashboard", get(admin_dashboard))
        .route("/admin/users", get(list_users))
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
    use mongodb::bson::doc;

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
