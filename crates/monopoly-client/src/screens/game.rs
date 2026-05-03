use std::sync::mpsc;
use macroquad::prelude::*;
use monopoly_core::network::ClientMessage;
use monopoly_core::state::GameState;
use crate::screens::Screen;
use crate::theme::Theme;

pub struct GameScreen {
    pub state: GameState,
    pub my_id: u8,
    pub tx: mpsc::Sender<ClientMessage>,
    pub theme: Theme,
    pub bid_input: String,
    pub event_log: Vec<String>,
}

impl GameScreen {
    pub fn new(state: GameState, my_id: u8, tx: mpsc::Sender<ClientMessage>, theme: Theme) -> Self {
        Self { state, my_id, tx, theme, bid_input: String::new(), event_log: Vec::new() }
    }

    pub fn push_event(&mut self, msg: String) {
        self.event_log.push(msg);
    }

    pub fn update(&mut self) -> Option<Screen> {
        None
    }

    pub fn draw(&self) {
        let t = &self.theme;
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), t.window_bg);
        draw_text("GAME - stub", 100.0, 100.0, 32.0, t.panel_text);
    }
}
