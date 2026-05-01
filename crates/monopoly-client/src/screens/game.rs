use std::sync::mpsc;
use macroquad::prelude::*;
use monopoly_core::network::ClientMessage;
use monopoly_core::state::GameState;
use crate::screens::Screen;

pub struct GameScreen {
    pub state: GameState,
    pub my_id: u8,
    pub tx: mpsc::Sender<ClientMessage>,
}

impl GameScreen {
    pub fn new(state: GameState, my_id: u8, tx: mpsc::Sender<ClientMessage>) -> Self {
        Self { state, my_id, tx }
    }

    pub fn update(&mut self) -> Option<Screen> {
        None
    }

    pub fn draw(&self) {
        draw_text("GAME", 580.0, 100.0, 48.0, BLACK);
        draw_text(
            &format!("Turn: {}", self.state.turn_number),
            50.0, 50.0, 24.0, BLACK,
        );
    }
}
