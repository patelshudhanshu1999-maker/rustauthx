mod database;
mod routes;

use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    // 1️⃣ Connect MongoDB (HARD GATE)
    let mongo_client = database::connect().await?;

    // 2️⃣ Build router
    let app = routes::create_router(mongo_client);

    // 3️⃣ Bind TCP listener
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server running on http://{}", addr);

    // 4️⃣ Start server
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
