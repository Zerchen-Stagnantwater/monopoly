use std::sync::mpsc;
use macroquad::prelude::*;
use monopoly_core::network::ClientMessage;
use monopoly_core::player::Token;
use crate::screens::Screen;
use crate::theme::Theme;

pub struct LobbyScreen {
    pub tx: mpsc::Sender<ClientMessage>,
    pub theme: Theme,
    pub my_id: Option<u8>,
    pub players: Vec<(u8, String, Token)>,
    pub name: String,
    joined: bool,
}

impl LobbyScreen {
    pub fn new(tx: mpsc::Sender<ClientMessage>, name: String, theme: Theme) -> Self {
        Self { tx, theme, my_id: None, players: Vec::new(), name, joined: false }
    }

    pub fn update(&mut self) -> Option<Screen> {
        if !self.joined {
            let _ = self.tx.send(ClientMessage::Join {
                name: self.name.clone(),
                token: Token::Car,
            });
            self.joined = true;
        }
        if is_key_pressed(KeyCode::S) {
            let _ = self.tx.send(ClientMessage::StartGame);
        }
        None
    }
    pub fn draw(&self) {
        let t = &self.theme;
        let cx = screen_width() / 2.0;

        // Window background already cleared in main loop
        // Panel
        draw_rectangle(cx - 300.0, 60.0, 600.0, 680.0, t.panel_bg);
        draw_rectangle_lines(cx - 300.0, 60.0, 600.0, 680.0, 2.0, t.panel_border);

        // Title
        draw_text("LOBBY", cx - 60.0, 120.0, t.title_size, t.money_color);
        draw_line(cx - 260.0, 138.0, cx + 260.0, 138.0, 1.0, t.panel_border);

        // My ID debug
        if let Some(id) = self.my_id {
            draw_text(
                &format!("You are Player {}", id),
                cx - 260.0, 165.0, t.small_size, t.panel_subtext,
                );
        }

        draw_text("PLAYERS", cx - 260.0, 195.0, t.label_size, t.panel_subtext);

        if self.players.is_empty() {
            draw_text("Waiting for players...", cx - 260.0, 230.0, t.body_size, t.panel_subtext);
        }

        for (i, (id, name, _)) in self.players.iter().enumerate() {
            let y = 220.0 + i as f32 * 55.0;
            let color = t.player_colors[*id as usize % 6];

            // Player row background
            draw_rectangle(cx - 260.0, y, 520.0, 44.0, t.input_bg);
            draw_rectangle_lines(cx - 260.0, y, 520.0, 44.0, 1.0, t.panel_border);

            // Color dot
            draw_circle(cx - 232.0, y + 22.0, 12.0, color);

            // Name — explicitly white so it's always visible on dark bg
            draw_text(
                &format!("Player {}  —  {}", id, name),
                cx - 210.0, y + 28.0, t.body_size, t.panel_text,
            );
        }

        draw_line(cx - 260.0, 620.0, cx + 260.0, 620.0, 1.0, t.panel_border);

        // Start button — only host sees it, but show player count to everyone
        let active = self.players.len();
        draw_text(
            &format!("{}/6 players", active),
            cx - 260.0, 650.0, t.small_size, t.panel_subtext,
            );

        if self.my_id == Some(0) {
            if active >= 2 {
                draw_rectangle(cx - 100.0, 658.0, 200.0, 44.0, t.button_bg);
                draw_text("[S] START GAME", cx - 88.0, 686.0, t.body_size, t.button_text);
            } else {
                draw_text("Need 2+ players to start", cx - 130.0, 680.0, t.body_size, t.panel_subtext);
            }
        } else {
            draw_text("Waiting for host to start...", cx - 140.0, 680.0, t.body_size, t.panel_subtext);
        }
    }
}
