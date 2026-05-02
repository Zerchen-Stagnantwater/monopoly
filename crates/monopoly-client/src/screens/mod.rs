mod connect;
mod lobby;
mod game;
mod gameover;

use std::sync::mpsc;
use macroquad::prelude::Color;
use monopoly_core::network::{ClientMessage, ServerMessage};
//use monopoly_core::player::Token;
use crate::theme::Theme;

pub use connect::ConnectScreen;
pub use lobby::LobbyScreen;
pub use game::GameScreen;
pub use gameover::GameOverScreen;

pub enum Screen {
    Connect(ConnectScreen),
    Lobby(LobbyScreen),
    Game(GameScreen),
    GameOver(GameOverScreen),
}

impl Screen {
    pub fn connect(tx: mpsc::Sender<ClientMessage>, theme: Theme) -> Self {
        Screen::Connect(ConnectScreen::new(tx, theme))
    }

    pub fn window_bg(&self) -> Color {
        match self {
            Screen::Connect(s)  => s.theme.window_bg,
            Screen::Lobby(s)    => s.theme.window_bg,
            Screen::Game(s)     => s.theme.window_bg,
            Screen::GameOver(s) => s.theme.window_bg,
        }
    }

    pub fn update(&mut self, rx: &mpsc::Receiver<ServerMessage>) {
        while let Ok(msg) = rx.try_recv() {
            self.handle_message(msg);
        }

        let next = match self {
            Screen::Connect(s)  => s.update(),
            Screen::Lobby(s)    => s.update(),
            Screen::Game(s)     => s.update(),
            Screen::GameOver(s) => s.update(),
        };

        if let Some(next_screen) = next {
            *self = next_screen;
        }
    }

    pub fn draw(&self) {
        match self {
            Screen::Connect(s)  => s.draw(),
            Screen::Lobby(s)    => s.draw(),
            Screen::Game(s)     => s.draw(),
            Screen::GameOver(s) => s.draw(),
        }
    }

    fn handle_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::JoinAck { assigned_id } => {
                if let Screen::Lobby(s) = self {
                    s.my_id = Some(assigned_id);
                }
            }
            ServerMessage::LobbyState { players } => {
                if let Screen::Lobby(s) = self {
                    s.players = players;
                }
            }
            ServerMessage::PlayerJoined { id, name, token } => {
                if let Screen::Lobby(s) = self {
                    s.players.push((id, name, token));
                }
            }
            ServerMessage::GameStarted => {}
            ServerMessage::StateUpdate { state } => {
                match self {
                    Screen::Lobby(s) => {
                        *self = Screen::Game(GameScreen::new(
                            state,
                            s.my_id.unwrap_or(0),
                            s.tx.clone(),
                            s.theme.clone(),
                        ));
                    }
                    Screen::Game(s) => {
                        s.state = state;
                    }
                    _ => {}
                }
            }
            ServerMessage::GameOver { winner_id, winner_name } => {
                let theme = match self {
                    Screen::Game(s) => s.theme.clone(),
                    _ => crate::theme::classic_theme(),
                };
                *self = Screen::GameOver(GameOverScreen::new(winner_id, winner_name, theme));
            }
            _ => {}
        }
    }
}
