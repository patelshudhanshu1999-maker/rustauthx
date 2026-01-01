use mongodb::Client;

pub async fn connect() -> mongodb::error::Result<Client> {
    let mongo_client = Client::with_uri_str("mongodb://localhost:27017").await?;
    println!("✅ MongoDB connected");
    Ok(mongo_client)
}
