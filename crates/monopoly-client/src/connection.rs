use anyhow::Result;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use monopoly_core::network::{ClientMessage, ServerMessage, Packet};

/// A live connection to the server.
/// The GUI talks to this — send actions, receive state updates.
pub struct Connection {
    tx: mpsc::UnboundedSender<ClientMessage>,
    pub rx: mpsc::UnboundedReceiver<ServerMessage>,
}

impl Connection {
    /// Connect to a server at the given address (e.g. "192.168.1.5:7777")
    pub async fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let (mut reader, mut writer) = tokio::io::split(stream);

        // Outgoing: GUI → server
        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<ClientMessage>();
        // Incoming: server → GUI
        let (in_tx, in_rx) = mpsc::unbounded_channel::<ServerMessage>();

        // Writer task — takes ClientMessages and sends them over TCP
        tokio::spawn(async move {
            while let Some(msg) = out_rx.recv().await {
                match Packet::encode(&msg) {
                    Ok(bytes) => {
                        if writer.write_all(&bytes).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => tracing::warn!("Encode error: {}", e),
                }
            }
        });

        // Reader task — reads ServerMessages and forwards to GUI
        tokio::spawn(async move {
            loop {
                let mut len_buf = [0u8; 4];
                if reader.read_exact(&mut len_buf).await.is_err() {
                    tracing::info!("Disconnected from server");
                    break;
                }
                let len = u32::from_le_bytes(len_buf) as usize;
                let mut payload = vec![0u8; len];
                if reader.read_exact(&mut payload).await.is_err() {
                    break;
                }
                match Packet::decode::<ServerMessage>(&payload) {
                    Ok(msg) => {
                        if in_tx.send(msg).is_err() {
                            break;
                        }
                    }
                    Err(e) => tracing::warn!("Decode error: {}", e),
                }
            }
        });

        Ok(Self { tx: out_tx, rx: in_rx })
    }

    /// Send an action to the server.
    pub fn send(&self, msg: ClientMessage) {
        let _ = self.tx.send(msg);
    }

    /// Poll for the next message from the server (non-blocking).
    pub fn try_recv(&mut self) -> Option<ServerMessage> {
        self.rx.try_recv().ok()
    }
}
