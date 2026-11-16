pub mod routes;

use anyhow::Result;
use routes::create_router;

#[tokio::main]
async fn main() -> Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");
    println!("Starting Interlock API server on http://{addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, create_router()).await?;
    Ok(())
}
