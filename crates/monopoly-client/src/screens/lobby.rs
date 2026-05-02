use std::sync::mpsc;
use macroquad::prelude::*;
use monopoly_core::network::ClientMessage;
use monopoly_core::player::Token;
use crate::screens::Screen;

pub struct LobbyScreen {
    pub tx: mpsc::Sender<ClientMessage>,
    pub my_id: Option<u8>,
    pub players: Vec<(u8, String, Token)>,
    pub name: String,
    joined: bool,
}

impl LobbyScreen {
    pub fn new(tx: mpsc::Sender<ClientMessage>, name: String) -> Self {
        Self {
            tx,
            my_id: None,
            players: Vec::new(),
            name,
            joined: false,
        }
    }

    pub fn update(&mut self) -> Option<Screen> {
        // Send Join once we arrive at this screen
        if !self.joined {
            let _ = self.tx.send(ClientMessage::Join {
                name: self.name.clone(),
                token: Token::Car,
            });
            self.joined = true;
        }

        if is_key_pressed(KeyCode::S){
            let _ = self.tx.send(ClientMessage::StartGame);
        }

        None
    }

    pub fn draw(&self) {
        let cx = screen_width() / 2.0;
        draw_text("LOBBY", cx - 60.0, 80.0, 48.0, BLACK);
       // draw_text("Waiting for players...", cx - 120.0, 140.0, 24.0, DARKGRAY);
        if self.my_id == Some(0) {
            draw_text("[S] Start game", cx - 120.0, 140.0 , 24.0, DARKGREEN);
        } else {
            draw_text("Waiting for host to start...", cx, 140.0 , 24.0, DARKGRAY);
        }
        for (i, (id, name, _token)) in self.players.iter().enumerate() {
            draw_text(
                &format!("Player {}: {}", id, name),
                cx - 150.0,
                220.0 + i as f32 * 40.0,
                26.0,
                BLACK,
            );
        }

        draw_text("Host: press s to start", cx - 120.0, 600.0, 22.0, DARKGRAY);
    }
}
