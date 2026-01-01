use axum::{Extension, Json};
use bcrypt::{DEFAULT_COST, hash};
use mongodb::{
    Client,
    bson::{doc, oid::ObjectId},
};
use uuid::Uuid;

use crate::models::user::{RegisterRequest, RegisterResponse};

pub async fn register(
    Extension(client): Extension<Client>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, (axum::http::StatusCode, String)> {
    // 1️⃣ Basic validation
    if payload.email.is_empty() || payload.password.len() < 6 {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Invalid email or password".into(),
        ));
    }

    // 2️⃣ Hash password
    let hashed_password = hash(payload.password, DEFAULT_COST).map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Hashing failed".into(),
        )
    })?;

    // 3️⃣ Insert into Mongo
    let db = client.database("rustauthx");
    let users = db.collection("users");

    let user_id = Uuid::new_v4().to_string();

    users
        .insert_one(
            doc! {
                "_id": &user_id,
                "email": &payload.email,
                "password": hashed_password,
            },
            None,
        )
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "DB insert failed".into(),
            )
        })?;

    // 4️⃣ Response
    Ok(Json(RegisterResponse {
        id: user_id,
        email: payload.email,
    }))
}
