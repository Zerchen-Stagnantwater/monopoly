use std::sync::mpsc;
use monopoly_core::network::{ClientMessage, ServerMessage};
use crate::connection::Connection;

pub fn run(
    net_tx: mpsc::Sender<ServerMessage>,
    action_rx: mpsc::Receiver<ClientMessage>,
) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async move {
        // Wait for the connect address from the main thread
        // The first ClientMessage is always a special Connect signal
        let addr = match action_rx.recv() {
            Ok(ClientMessage::Connect { addr }) => addr,
            _ => return,
        };

        let mut conn = match Connection::connect(&addr).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to connect: {}", e);
                return;
            }
        };

        loop {
            // Forward any pending actions from GUI to server
            while let Ok(msg) = action_rx.try_recv() {
                conn.send(msg);
            }

            // Forward any incoming server messages to GUI
            while let Some(msg) = conn.try_recv() {
                if net_tx.send(msg).is_err() {
                    return; // GUI shut down
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
        }
    });
}
