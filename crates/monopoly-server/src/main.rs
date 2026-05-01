mod lobby;
mod session;
mod engine;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use lobby::Lobby;

pub const DEFAULT_PORT: u16 = 7777;
pub const MAX_PLAYERS: u8 = 6;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = format!("0.0.0.0:{}", DEFAULT_PORT);
    let listener = TcpListener::bind(&addr).await?;
    info!("monopoly-server listening on {}", addr);
    info!("Other players connect to your public IP on port {}", DEFAULT_PORT);

    let lobby = Arc::new(Mutex::new(Lobby::new()));

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("New connection from {}", addr);

        let lobby = Arc::clone(&lobby);
        tokio::spawn(async move {
            if let Err(e) = session::handle(socket, lobby).await {
                tracing::warn!("Session error: {}", e);
            }
        });
    }
}
