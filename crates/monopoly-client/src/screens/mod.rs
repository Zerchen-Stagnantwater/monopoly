mod connect;
mod lobby;
mod game;
mod gameover;

use std::sync::mpsc;
use monopoly_core::network::{ClientMessage, ServerMessage};

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
    pub fn connect(tx: mpsc::Sender<ClientMessage>) -> Self {
        Screen::Connect(ConnectScreen::new(tx))
    }

    pub fn update(&mut self, rx: &mpsc::Receiver<ServerMessage>) {
        // Drain incoming server messages
        while let Ok(msg) = rx.try_recv() {
            self.handle_message(msg);
        }

        // Update current screen
        let next = match self {
            Screen::Connect(s)   => s.update(),
            Screen::Lobby(s)     => s.update(),
            Screen::Game(s)      => s.update(),
            Screen::GameOver(s)  => s.update(),
        };

        if let Some(next_screen) = next {
            *self = next_screen;
        }
    }

    pub fn draw(&self) {
        match self {
            Screen::Connect(s)   => s.draw(),
            Screen::Lobby(s)     => s.draw(),
            Screen::Game(s)      => s.draw(),
            Screen::GameOver(s)  => s.draw(),
        }
    }

    fn handle_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::JoinAck { assigned_id } => {
                if let Screen::Lobby(s) = self {
                    s.my_id = Some(assigned_id);
                }
            }
            ServerMessage::PlayerJoined { id, name, token } => {
                if let Screen::Lobby(s) = self {
                    s.players.push((id, name, token));
                }
            }
            ServerMessage::GameStarted => {
                // Transition handled on StateUpdate
            }
            ServerMessage::StateUpdate { state } => {
                match self {
                    Screen::Lobby(s) => {
                        // Game started — transition to game screen
                        *self = Screen::Game(GameScreen::new(state, s.my_id.unwrap_or(0), s.tx.clone()));
                    }
                    Screen::Game(s) => {
                        s.state = state;
                    }
                    _ => {}
                }
            }
            ServerMessage::GameOver { winner_id, winner_name } => {
                *self = Screen::GameOver(GameOverScreen::new(winner_id, winner_name));
            }
            _ => {
                // Handle in individual screens later
            }
        }
    }
}
