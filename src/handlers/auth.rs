use axum::{Extension, Json};
use bcrypt::{DEFAULT_COST, hash, verify};
#[allow(unused_imports)]
use mongodb::{Client, bson::doc};
use uuid::Uuid;

use crate::models::user::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse};

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

pub async fn login(
    Extension(client): Extension<Client>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (axum::http::StatusCode, String)> {
    // find the user by email
    let db = client.database("rustauthx");
    let users = db.collection::<mongodb::bson::Document>("users");

    let user = users
        .find_one(doc! {"email": &payload.email}, None)
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "DB find failed".into(),
            )
        })?;

    let user = match user {
        Some(u) => u,
        None => {
            return Err((
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid email or password".into(),
            ));
        }
    };

    let hashed_password = user.get_str("password").map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Corrupt user data".into(),
        )
    })?;

    let is_valid = verify(&payload.password, hashed_password).map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Hash error".into(),
        )
    })?;

    if !is_valid {
        return Err((
            axum::http::StatusCode::UNAUTHORIZED,
            "Invalid email or password".into(),
        ));
    }

    let user_id = user.get_str("_id").map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Missing user id".into(),
        )
    })?;

    Ok(Json(LoginResponse {
        user_id: user_id.to_string(),
        email: payload.email,
    }))
}
