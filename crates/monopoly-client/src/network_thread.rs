use std::sync::mpsc;
use monopoly_core::network::{ClientMessage, ServerMessage};
use crate::connection::Connection;

pub fn run(
    net_tx: mpsc::Sender<ServerMessage>,
    action_rx: mpsc::Receiver<ClientMessage>,
) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async move {
        let addr = loop {
            match action_rx.try_recv() {
                Ok(ClientMessage::Connect { addr }) => break addr,
                Ok(_) => {}
                Err(_) => {}
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        };

        let mut conn = match Connection::connect(&addr).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to connect: {}", e);
                return;
            }
        };

        loop {
            while let Ok(msg) = action_rx.try_recv() {
                conn.send(msg);
            }
            while let Some(msg) = conn.try_recv() {
                if net_tx.send(msg).is_err() {
                    return;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
        }
    });
}
