mod connection;

use anyhow::Result;
use tracing::info;
use monopoly_core::network::ClientMessage;
use monopoly_core::player::Token;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("monopoly-client starting up");

    // Temporary smoke test — connect, send Join, print responses
    // Will be replaced by raylib GUI in Phase 4
    let addr = std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:7777".to_string());
    info!("Connecting to {}", addr);

    let mut conn = connection::Connection::connect(&addr).await?;
    info!("Connected");

    conn.send(ClientMessage::Join {
        name: "TestPlayer".to_string(),
        token: Token::Car,
    });

    // Listen for a few messages then exit
    for _ in 0..5 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        while let Some(msg) = conn.try_recv() {
            info!("Received: {:?}", msg);
        }
    }

    Ok(())
}
