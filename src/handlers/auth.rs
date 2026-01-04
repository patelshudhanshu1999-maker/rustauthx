use axum::http::StatusCode;
use axum::{Extension, Json};
use bcrypt::{DEFAULT_COST, hash, verify};
use mongodb::{Client, bson::doc};
use uuid::Uuid;

use crate::auth::jwt::generate_token;
use crate::models::token::RefreshRequest;
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

    // let is_valid = verify(&payload.password, hashed_password).map_err(|_| {
    //     (
    //         axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    //         "Hash error".into(),
    //     )
    // })?;

    if !verify(&payload.password, hashed_password).unwrap() {
        return Err((
            axum::http::StatusCode::UNAUTHORIZED,
            "Invalid email or password".into(),
        ));
    }

    // if !is_valid {
    //     return Err((
    //         axum::http::StatusCode::UNAUTHORIZED,
    //         "Invalid email or password".into(),
    //     ));
    // }

    // Extract user_id BEFORE using it
    let user_id = user.get_str("_id").map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Missing user id".into(),
        )
    })?;

    // let now = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_secs();

    // let claims = Claims {
    //     sub: user_id.to_string(),
    //     exp: (now + JWT_EXP_HOURS as u64 * 3600) as usize,
    // };

    // let token = encode(&Header::default(), &claims, &encoding_key()).map_err(|_| {
    //     (
    //         axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    //         "Token creation failed".into(),
    //     )
    // })?;

    let token = generate_token(user_id);

    let refresh_token = Uuid::new_v4().to_string();

    let tokens = db.collection("refresh_tokens");
    tokens
        .insert_one(
            doc! {
                "user_id": &user_id,
                "token": &refresh_token,
                "access_token": &token,  // Store the access token
            },
            None,
        )
        .await
        .unwrap();

    Ok(Json(LoginResponse {
        token,
        refresh_token,
    }))
}

pub async fn refresh(
    Extension(client): Extension<Client>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, (axum::http::StatusCode, String)> {
    let db = client.database("rustauthx");
    let tokens = db.collection::<mongodb::bson::Document>("refresh_tokens");

    // Verify the refresh token exists and get the associated access token
    let stored = tokens
        .find_one(doc! {"token": &payload.refresh_token}, None)
        .await
        .unwrap()
        .ok_or((
            axum::http::StatusCode::UNAUTHORIZED,
            "Invalid refresh token".into(),
        ))?;

    let user_id = stored.get_str("user_id").unwrap();

    // Get the OLD access token from the database
    let old_access_token = stored.get_str("access_token").ok();

    // If there's an old access token, blacklist it
    if let Some(old_token) = old_access_token {
        println!("🔍 Old token from DB: {}", &old_token[..20]);

        // Decode the old token to get its expiration
        if let Ok(token_data) = jsonwebtoken::decode::<crate::models::claims::Claims>(
            old_token,
            &crate::auth::jwt::decoding_key(),
            &jsonwebtoken::Validation::default(),
        ) {
            // Blacklist the old token
            println!("🗑️  Blacklisting old token...");
            crate::handlers::blacklist::blacklist_token(
                &client,
                old_token,
                token_data.claims.exp as i64,
            )
            .await
            .map_err(|e| {
                println!("❌ Blacklist failed: {}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to blacklist token: {}", e),
                )
            })?;
            println!("✅ Token blacklisted successfully");
        }
    }

    // Generate new access token
    let new_access_token = generate_token(user_id);
    println!("✅ New token generated");

    // Update the refresh token document with the NEW access token
    tokens
        .update_one(
            doc! {"token": &payload.refresh_token},
            doc! {"$set": {"access_token": &new_access_token}},
            None,
        )
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update token".into(),
            )
        })?;

    Ok(Json(LoginResponse {
        token: new_access_token,
        refresh_token: payload.refresh_token,
    }))
}

pub async fn logout(
    Extension(client): Extension<Client>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, (StatusCode, String)> {
    // Extract access token from Authorization header
    let auth_header = headers.get("Authorization").ok_or((
        StatusCode::UNAUTHORIZED,
        "Authorization header required".into(),
    ))?;

    let auth_str = auth_header.to_str().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid Authorization header".into(),
        )
    })?;

    let access_token = auth_str.strip_prefix("Bearer ").ok_or((
        StatusCode::BAD_REQUEST,
        "Invalid Authorization format".into(),
    ))?;

    // Decode the access token to get user_id and expiration
    let token_data = jsonwebtoken::decode::<crate::models::claims::Claims>(
        access_token,
        &crate::auth::jwt::decoding_key(),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token".into()))?;

    let user_id = token_data.claims.sub;

    // Blacklist the access token
    crate::handlers::blacklist::blacklist_token(
        &client,
        access_token,
        token_data.claims.exp as i64,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to blacklist token: {}", e),
        )
    })?;
    println!("✅ Access token blacklisted on logout");

    // Delete all refresh tokens for this user
    let db = client.database("rustauthx");
    let tokens = db.collection::<mongodb::bson::Document>("refresh_tokens");

    tokens
        .delete_many(doc! {"user_id": &user_id}, None)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete refresh tokens".into(),
            )
        })?;

    println!("✅ All refresh tokens deleted for user: {}", user_id);
    Ok(StatusCode::OK)
}
