pub mod routes;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Interlock API placeholder (cut-list item 2 fallback to precomputed JSON)");
    let health = routes::handle_health();
    println!("Status: {} ({})", health.status, health.service);
    Ok(())
}
