use std::sync::mpsc;
use macroquad::prelude::*;
use monopoly_core::network::ClientMessage;
use monopoly_core::player::Token;
use crate::screens::Screen;

pub struct LobbyScreen {
    pub tx: mpsc::Sender<ClientMessage>,
    pub my_id: Option<u8>,
    pub players: Vec<(u8, String, Token)>,
}

impl LobbyScreen {
    pub fn new(tx: mpsc::Sender<ClientMessage>) -> Self {
        Self { tx, my_id: None, players: Vec::new() }
    }

    pub fn update(&mut self) -> Option<Screen> {
        None
    }

    pub fn draw(&self) {
        draw_text("LOBBY", 580.0, 100.0, 48.0, BLACK);
        for (i, (id, name, _token)) in self.players.iter().enumerate() {
            draw_text(
                &format!("Player {}: {}", id, name),
                400.0,
                200.0 + i as f32 * 40.0,
                28.0,
                BLACK,
            );
        }
    }
}
