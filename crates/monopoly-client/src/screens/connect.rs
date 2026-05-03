use std::sync::mpsc;
use macroquad::prelude::*;
use monopoly_core::network::ClientMessage;
use crate::screens::{Screen, lobby::LobbyScreen};
use crate::theme::Theme;

pub struct ConnectScreen {
    pub tx: mpsc::Sender<ClientMessage>,
    pub theme: Theme,
    addr_input: String,
    name_input: String,
    active_field: Field,
    error: Option<String>,
}

#[derive(PartialEq)]
enum Field { Address, Name }

impl ConnectScreen {
    pub fn new(tx: mpsc::Sender<ClientMessage>, theme: Theme) -> Self {
        Self {
            tx,
            theme,
            addr_input: "127.0.0.1:7777".to_string(),
            name_input: String::new(),
            active_field: Field::Address,
            error: None,
        }
    }

pub fn update(&mut self) -> Option<Screen> {
    if is_key_pressed(KeyCode::Tab) {
        self.active_field = match self.active_field {
            Field::Address => Field::Name,
            Field::Name => Field::Address,
        };
    }

    // Check Enter BEFORE get_char_pressed so the \n isn't consumed into a field
    if is_key_pressed(KeyCode::Enter) {
        if self.name_input.trim().is_empty() {
            self.error = Some("Please enter your name".to_string());
        } else {
            let addr = self.addr_input
                .chars()
                .filter(|c| c.is_ascii_graphic())
                .collect::<String>();
            let _ = self.tx.send(ClientMessage::Connect { addr: addr.clone() });
            return Some(Screen::Lobby(LobbyScreen::new(
                self.tx.clone(),
                self.name_input.trim().to_string(),
                self.theme.clone(),
            )));
        }
        return None;
    }

    if let Some(c) = get_char_pressed() {
        if c != '\t' && c != '\r' && c != '\n' {
            match self.active_field {
                Field::Address => self.addr_input.push(c),
                Field::Name => self.name_input.push(c),
            }
        }
    }

    if is_key_pressed(KeyCode::Backspace) {
        match self.active_field {
            Field::Address => { self.addr_input.pop(); }
            Field::Name => { self.name_input.pop(); }
        }
    }

    None
}
    pub fn draw(&self) {
        let t = &self.theme;
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;

        // Title
        draw_text("MONOPOLY", cx - 160.0, cy - 180.0, t.title_size * 1.5, t.money_color);
        draw_line(cx - 220.0, cy - 150.0, cx + 220.0, cy - 150.0, 2.0, t.panel_border);

        // Address field
        let addr_col = if self.active_field == Field::Address { t.input_border_active } else { t.input_border_inactive };
        draw_text("SERVER ADDRESS", cx - 200.0, cy - 110.0, t.small_size, t.panel_subtext);
        draw_rectangle(cx - 200.0, cy - 100.0, 400.0, 40.0, t.input_bg);
        draw_rectangle_lines(cx - 200.0, cy - 100.0, 400.0, 40.0, 2.0, addr_col);
        draw_text(&self.addr_input, cx - 188.0, cy - 72.0, t.body_size, t.panel_text);

        // Name field
        let name_col = if self.active_field == Field::Name { t.input_border_active } else { t.input_border_inactive };
        draw_text("YOUR NAME", cx - 200.0, cy - 20.0, t.small_size, t.panel_subtext);
        draw_rectangle(cx - 200.0, cy - 10.0, 400.0, 40.0, t.input_bg);
        draw_rectangle_lines(cx - 200.0, cy - 10.0, 400.0, 40.0, 2.0, name_col);
        draw_text(&self.name_input, cx - 188.0, cy + 18.0, t.body_size, t.panel_text);

        // Hint
        draw_text("TAB to switch  |  ENTER to connect", cx - 160.0, cy + 60.0, t.small_size, t.panel_subtext);

        // Error
        if let Some(err) = &self.error {
            draw_text(err, cx - 160.0, cy + 90.0, t.body_size, t.error_color);
        }
    }
}
