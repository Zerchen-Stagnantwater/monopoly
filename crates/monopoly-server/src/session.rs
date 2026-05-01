use anyhow::Result;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, mpsc};
use std::sync::Arc;
use tracing::info;
use monopoly_core::network::{ClientMessage, ServerMessage, Packet};
use crate::lobby::Lobby;

pub async fn handle(mut socket: TcpStream, lobby: Arc<Mutex<Lobby>>) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    let mut player_id: Option<u8> = None;

    loop {
        let mut len_buf = [0u8; 4];
        tokio::select! {
            // Incoming from client
            result = socket.read_exact(&mut len_buf) => {
                match result {
                    Ok(_) => {
                        let len = u32::from_le_bytes(len_buf) as usize;
                        let mut payload = vec![0u8; len];
                        socket.read_exact(&mut payload).await?;

                        let msg: ClientMessage = Packet::decode(&payload)?;
                        let response = crate::engine::handle_message(
                            &lobby,
                            player_id,
                            msg,
                            tx.clone(),
                        ).await;

                        match response {
                            Ok(Some(id)) => player_id = Some(id),
                            Ok(None) => {}
                            Err(e) => {
                                let reject = ServerMessage::ActionRejected {
                                    reason: e.to_string(),
                                };
                                let bytes = Packet::encode(&reject)?;
                                socket.write_all(&bytes).await?;
                            }
                        }
                    }
                    Err(_) => {
                        info!("Client disconnected");
                        if let Some(id) = player_id {
                            let mut lobby = lobby.lock().await;
                            lobby.remove_player(id);
                            lobby.broadcast(ServerMessage::PlayerLeft { id });
                        }
                        break;
                    }
                }
            }

            // Outgoing to client
            Some(msg) = rx.recv() => {
                let bytes = Packet::encode(&msg)?;
                socket.write_all(&bytes).await?;
            }
        }
    }

    Ok(())
}
