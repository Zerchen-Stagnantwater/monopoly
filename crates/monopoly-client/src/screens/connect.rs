use std::sync::mpsc;
use macroquad::prelude::*;
use monopoly_core::network::ClientMessage;
use crate::screens::Screen;

pub struct ConnectScreen {
    pub tx: mpsc::Sender<ClientMessage>,
    pub addr_input: String,
}

impl ConnectScreen {
    pub fn new(tx: mpsc::Sender<ClientMessage>) -> Self {
        Self {
            tx,
            addr_input: "127.0.0.1:7777".to_string(),
        }
    }

    pub fn update(&mut self) -> Option<Screen> {
        None
    }

    pub fn draw(&self) {
        draw_text("MONOPOLY", 500.0, 200.0, 60.0, BLACK);
        draw_text("Enter server address and press Enter", 400.0, 300.0, 24.0, DARKGRAY);
        draw_text(&self.addr_input, 500.0, 360.0, 28.0, BLACK);
    }
}
