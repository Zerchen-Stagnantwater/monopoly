use std::sync::mpsc;
use macroquad::prelude::*;
use monopoly_core::network::ClientMessage;
use crate::screens::{Screen, lobby::LobbyScreen};

pub struct ConnectScreen {
    pub tx: mpsc::Sender<ClientMessage>,
    addr_input: String,
    name_input: String,
    active_field: Field,
    error: Option<String>,
}

#[derive(PartialEq)]
enum Field {
    Address,
    Name,
}

impl ConnectScreen {
    pub fn new(tx: mpsc::Sender<ClientMessage>) -> Self {
        Self {
            tx,
            addr_input: "127.0.0.1:7777".to_string(),
            name_input: String::new(),
            active_field: Field::Address,
            error: None,
        }
    }

    pub fn update(&mut self) -> Option<Screen> {
        // Tab switches fields
        if is_key_pressed(KeyCode::Tab) {
            self.active_field = match self.active_field {
                Field::Address => Field::Name,
                Field::Name => Field::Address,
            };
        }

        // Type into active field
        if let Some(c) = get_char_pressed() {
            if c != '\t' && c != '\r' && c != '\n' {
                match self.active_field {
                    Field::Address => self.addr_input.push(c),
                    Field::Name => self.name_input.push(c),
                }
            }
        }

        // Backspace
        if is_key_pressed(KeyCode::Backspace) {
            match self.active_field {
                Field::Address => { self.addr_input.pop(); }
                Field::Name => { self.name_input.pop(); }
            }
        }

        // Enter to connect
        if is_key_pressed(KeyCode::Enter) {
            if self.name_input.trim().is_empty() {
                self.error = Some("Please enter your name".to_string());
                return None;
            }
            if self.addr_input.trim().is_empty() {
                self.error = Some("Please enter a server address".to_string());
                return None;
            }

            // Signal network thread to connect
            let _ = self.tx.send(ClientMessage::Connect {
                addr: self.addr_input.trim().to_string(),
            });

            // Transition to lobby screen
            return Some(Screen::Lobby(LobbyScreen::new(
                self.tx.clone(),
                self.name_input.trim().to_string(),
            )));
        }

        None
    }

    pub fn draw(&self) {
        let cx = screen_width() / 2.0;

        draw_text("MONOPOLY", cx - 140.0, 180.0, 72.0, BLACK);
        draw_line(cx - 200.0, 210.0, cx + 200.0, 210.0, 2.0, DARKGRAY);

        // Address field
        let addr_color = if self.active_field == Field::Address { BLUE } else { DARKGRAY };
        draw_text("Server Address", cx - 200.0, 290.0, 22.0, DARKGRAY);
        draw_rectangle_lines(cx - 200.0, 300.0, 400.0, 40.0, 2.0, addr_color);
        draw_text(&self.addr_input, cx - 190.0, 327.0, 24.0, BLACK);

        // Name field
        let name_color = if self.active_field == Field::Name { BLUE } else { DARKGRAY };
        draw_text("Your Name", cx - 200.0, 380.0, 22.0, DARKGRAY);
        draw_rectangle_lines(cx - 200.0, 390.0, 400.0, 40.0, 2.0, name_color);
        draw_text(&self.name_input, cx - 190.0, 417.0, 24.0, BLACK);

        // Hint
        draw_text("Tab to switch fields  |  Enter to connect", cx - 180.0, 460.0, 18.0, GRAY);

        // Error
        if let Some(err) = &self.error {
            draw_text(err, cx - 180.0, 500.0, 20.0, RED);
        }
    }
}
