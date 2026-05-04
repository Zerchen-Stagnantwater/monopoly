use monopoly_core::player::Token;
use monopoly_core::{GameState, RuleSet};
use monopoly_core::load_board;
use std::collections::HashMap;
use tokio::sync::mpsc;
use monopoly_core::network::ServerMessage;

pub type ClientTx = mpsc::UnboundedSender<ServerMessage>;

#[derive(Debug, Clone)]
pub struct LobbyPlayer {
    pub id: u8,
    pub name: String,
    pub token: Token,
    #[allow(dead_code)]
    pub ready: bool,
    pub tx: ClientTx,
}

#[derive(Debug, PartialEq)]
pub enum LobbyState {
    WaitingForPlayers,
    InGame,
}

pub struct Lobby {
    pub players: HashMap<u8, LobbyPlayer>,
    pub state: LobbyState,
    pub game: Option<GameState>,
    pub ruleset: RuleSet,
    pub next_id: u8,
    pub host_id: Option<u8>,
    pub pending_trade: Option<monopoly_core::trading::TradeOffer>,
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            state: LobbyState::WaitingForPlayers,
            game: None,
            ruleset: RuleSet::standard(),
            next_id: 0,
            host_id: None,
            pending_trade: None,
        }
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= crate::MAX_PLAYERS as usize
    }

    pub fn add_player(&mut self, name: String, token: Token, tx: ClientTx) -> Option<u8> {
        if self.is_full() || self.state == LobbyState::InGame {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;

        if self.host_id.is_none() {
            self.host_id = Some(id);
        }

        self.players.insert(id, LobbyPlayer {
            id,
            name,
            token,
            ready: false,
            tx,
        });

        Some(id)
    }

    pub fn remove_player(&mut self, id: u8) {
        self.players.remove(&id);
        if self.host_id == Some(id) {
            self.host_id = self.players.keys().next().copied();
        }
    }

    pub fn broadcast(&self, msg: ServerMessage) {
        for player in self.players.values() {
            let _ = player.tx.send(msg.clone());
        }
    }

    pub fn send_to(&self, id: u8, msg: ServerMessage) {
        if let Some(player) = self.players.get(&id) {
            let _ = player.tx.send(msg);
        }
    }

    pub fn start_game(&mut self) -> anyhow::Result<()> {
        let board = load_board("config/boards/standard.toml")?;
        let card_decks = monopoly_core::load_card_decks("config/cards/standard.toml")?;
        let ruleset = &self.ruleset;

        let players = self.players.values().map(|lp| {
            monopoly_core::Player::new(
                lp.id,
                lp.name.clone(),
                lp.token.clone(),
                ruleset.starting_money,
            )
        }).collect();

        self.game = Some(GameState {
            board,
            players,
            current_player_index: 0,
            turn_phase: monopoly_core::state::TurnPhase::WaitingForRoll,
            turn_number: 0,
            last_roll: None,
            free_parking_pot: 0,
            game_over: false,
            winner: None,
            houses_remaining: ruleset.max_houses,
            hotels_remaining: ruleset.max_hotels,
            auction_passers: Vec::new(),
            card_decks,
            last_card: None,
        });

        self.state = LobbyState::InGame;
        Ok(())
    }
}
