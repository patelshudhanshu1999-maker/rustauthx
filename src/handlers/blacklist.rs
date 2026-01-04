use mongodb::{Client, bson::doc};
use std::time::{SystemTime, UNIX_EPOCH};

/// Add a token to the blacklist
pub async fn blacklist_token(client: &Client, token: &str, expires_at: i64) -> Result<(), String> {
    let db = client.database("rustauthx");
    let blacklist = db.collection::<mongodb::bson::Document>("token_blacklist");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    blacklist
        .insert_one(
            doc! {
                "_id": token,
                "blacklisted_at": now,
                "expires_at": expires_at,
            },
            None,
        )
        .await
        .map_err(|_| "Failed to blacklist token".to_string())?;

    Ok(())
}

/// Check if a token is blacklisted
pub async fn is_token_blacklisted(client: &Client, token: &str) -> Result<bool, String> {
    let db = client.database("rustauthx");
    let blacklist = db.collection::<mongodb::bson::Document>("token_blacklist");

    let result = blacklist
        .find_one(doc! {"_id": token}, None)
        .await
        .map_err(|_| "Failed to check blacklist".to_string())?;

    Ok(result.is_some())
}
