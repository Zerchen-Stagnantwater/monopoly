use anyhow::Result;
use tracing::info;
use monopoly_core::load_board;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("monopoly-server starting up");

    let board = load_board("config/boards/standard.toml")?;
    info!("Board loaded — {} tiles", board.tile_count());

    Ok(())
}
